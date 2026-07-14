use crate::{
    cas::{self, login::CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::{client, obs, request::cookie_parser},
};
use reqwest::{
    StatusCode, Url,
    header::{AUTHORIZATION, COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};
use serde_json::Value;

// AI 系统的认证架构与其他系统不同，走的是 OAuth2 Authorization Code 流程：
//
//   deepseek.hnu (OAuth2 authorize) → CAS (身份认证) → deepseek.hnu (签发 code) → maas (换 bearer token)
//
// 其他系统只需一步 CAS 直连：cas_token.get_ticket_url(固定URL)
// 因为那些系统本身就是 CAS client，CAS 认证后 ticket 直接回到目标系统即可。
// 而 AI 系统的认证入口是 deepseek.hnu 的 OAuth2 authorize endpoint，CAS 只是其身份认证的后端
// OAuth2 的 code 必须由 deepseek.hnu 签发，无法跳过 deepseek.hnu 直接构造 CAS URL。

// OAuth2 authorization endpoint
const INITIAL_AUTH_URL: &str = "https://deepseek.hnu.edu.cn:5556/auth?client_id=openclaw&response_type=code&redirect_uri=https%3A%2F%2Fmaas.nscc-cs.cn%2Fcallback&scope=openid%20profile%20email";
// maas 平台 OAuth2 token endpoint，用 authorization code 交换 bearer token
const OAUTH_LOGIN_URL: &str = "https://maas.nscc-cs.cn/api/oauth-login";

/// AI 系统的令牌
#[derive(Debug, Clone)]
pub struct AiToken {
    headers: HeaderMap,
}

/// 将 302 重定向的 `Location` 解析为绝对 URL（`Location` 可能是**绝对路径**也可能是**相对路径**）
fn resolve_location(current_url: &str, location: &str) -> Result<String, String> {
    // 已经是绝对 URL，直接使用
    if location.starts_with("http://") || location.starts_with("https://") {
        return Ok(location.to_string());
    }
    // 不支持不以 / 开头的相对路径
    if !location.starts_with('/') {
        return Err(format!("Location 不是绝对路径且没有前导 /: {}", location));
    }
    // 从 current_url 提取 origin，拼接 location
    let url = Url::parse(current_url).map_err(|e| format!("无效的 current_url: {}", e))?;
    Ok(format!(
        "{}://{}{}",
        url.scheme(),
        url.authority(),
        location
    ))
}

/// 从 URL query string 中提取 `code` 参数
fn extract_code(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    parsed
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
}

impl AiToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Arguments
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::acquire_by_login] 创建。
    ///
    /// # Returns
    ///
    /// 返回一个 [AiToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于当前 [CasToken] 过期导致登录失败，此时会返回 [cas::error::TokenExpired] 错误
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(cas_token), fields(subsystem = "ai", redirect_count = tracing::field::Empty), err)
    )]
    pub async fn acquire_by_cas_login(
        cas_token: &CasToken,
    ) -> Result<Self, crate::Error<cas::error::TokenExpired>> {
        let mut current_url = INITIAL_AUTH_URL.to_string();
        let mut all_cookies = cas_token.cookie().to_string();
        let mut cas_authenticated = false;

        // Phase 1: 跟随 OAuth2 authorize 链路的 302/303 重定向，直到抵达 maas callback。
        // 不能手动构造 CAS URL 然后直接 get_ticket_url ，那样即使 CAS 认证成功，
        // ticket 也不知道该回到哪里去（没有 OAuth2 authorize endpoint 来接收并签发 code）。
        // 所以这里必须从 INITIAL_AUTH_URL 开始，沿着服务器返回的 Location 自动跟到底。
        for _attempt in 0..10 {
            obs::debug!(attempt = _attempt, %current_url, %cas_authenticated, "oauth_redirect");
            let res = client
                .get(&current_url)
                .header(COOKIE, &all_cookies)
                .send()
                .await
                .network_err()?
                .error_for_status()
                .unexpected_err()?;

            let status = res.status();

            let new_cookies = cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ");
            if !new_cookies.is_empty() {
                if !all_cookies.is_empty() {
                    all_cookies.push_str("; ");
                }
                all_cookies.push_str(&new_cookies);
            }

            if status == StatusCode::FOUND || status == StatusCode::SEE_OTHER {
                let location = res
                    .headers()
                    .get(LOCATION)
                    .ok_or("无法获取重定向 Location")
                    .unexpected_err()?
                    .to_str()
                    .unexpected_err()?;
                obs::debug!(location = %location, "resolved_location");
                current_url = resolve_location(&current_url, location).unexpected_err()?;
            } else if current_url.contains("maas.nscc-cs.cn") {
                obs::debug!("reached_maas_callback");
                obs::record!(redirect_count = _attempt);
                break;
            } else if current_url.contains("cas.hnu.edu.cn") && !cas_authenticated {
                // OAuth2 authorize 链路中遇到了 CAS 登录页（HTTP 200），说明 deepseek.hnu
                // 把浏览器重定向到了 CAS 做身份认证。这里就是整个流程中 CAS 介入的节点：
                // 用 cas_token 向 CAS 证明身份，换取 ticket，然后继续 OAuth2 链路。
                //
                // 注意：不能使用沿途积累的 all_cookies，因为 CAS 返回的 200 登录页面
                // 会在 Set-Cookie 中写入一个新的、未认证的 JSESSIONID ，这个新 session
                // 会覆盖掉 cas_token 中已认证的 TGT cookie，导致 get_ticket_url 拿到的
                // 不是已登录用户的 ticket，进而返回 TokenExpired。所以必须用原始的
                // cas_token cookie 来做这一步。
                obs::debug!("cas_authentication_required");
                let ticket_url = cas_token.get_ticket_url(&current_url).await?;
                current_url = ticket_url;
                cas_authenticated = true;
            } else {
                let body = res.text().await.unwrap_or_default();
                obs::error!(status = %status, %current_url, body = %body, "unexpected_status");
                Err(format!(
                    "未预期的HTTP状态码 {}，URL: {}",
                    status, current_url
                ))
                .unexpected_err()?;
            }
        }

        if extract_code(&current_url).is_none() {
            Err(format!(
                "重定向次数超过限制，无法完成登录，当前URL: {}",
                current_url
            ))
            .unexpected_err()?;
        }

        // Phase 2: 从 callback URL 提取 code，POST 到 oauth-login 换 token
        let code = extract_code(&current_url)
            .ok_or_else(|| format!("无法从URL提取code: {}", current_url))
            .unexpected_err()?;
        obs::debug!(code = %code, "extracted_code");

        let oauth_res = client
            .post(OAUTH_LOGIN_URL)
            .header(COOKIE, &all_cookies)
            .json(&serde_json::json!({"code": code}))
            .send()
            .await
            .network_err()?;

        let session_cookies = cookie_parser(oauth_res.headers().get_all(SET_COOKIE)).join("; ");
        if !session_cookies.is_empty() {
            if !all_cookies.is_empty() {
                all_cookies.push_str("; ");
            }
            all_cookies.push_str(&session_cookies);
        }

        let oauth_text = oauth_res.text().await.unexpected_err()?;
        let oauth_json: Value = serde_json::from_str(&oauth_text).parse_err(&oauth_text)?;
        let bearer_token = oauth_json["data"]["token"]
            .as_str()
            .ok_or_else(|| parse_err(&oauth_text))?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", bearer_token)
                .parse()
                .unexpected_err()?,
        );
        headers.insert(COOKIE, all_cookies.parse().unexpected_err()?);
        Ok(Self { headers })
    }

    /// 从 [HeaderMap] 创建 [AiToken]
    ///
    /// # Arguments
    ///
    /// - `headers`: 一个合法的可用作 [AiToken] 的 [HeaderMap]
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [AiToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }

    /// 获取当前令牌的 [HeaderMap]，可用于 [AiToken::from_headers_unchecked]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use crate::{ai::test::get_ai_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_ai_token() -> TestResult<()> {
        let ai_token = get_ai_token().await?;
        println!("{:#?}", ai_token);
        Ok(())
    }
}
