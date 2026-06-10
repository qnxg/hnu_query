use crate::{
    cas::{error::TokenExpired, login::CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::{client, request::cookie_parser},
};
use reqwest::{
    StatusCode, Url,
    header::{AUTHORIZATION, COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};
use serde_json::Value;

// 登录入口，OAuth2 authorize 端点
const INITIAL_AUTH_URL: &str = "https://deepseek.hnu.edu.cn:5556/auth?client_id=openclaw&response_type=code&redirect_uri=https%3A%2F%2Fmaas.nscc-cs.cn%2Fcallback&scope=openid%20profile%20email";
// maas 平台 OAuth2 token 端点，通过 code 得到 bearer token
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
    url.split('?').nth(1)?.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == "code" {
            parts.next().map(|s| s.to_string())
        } else {
            None
        }
    })
}

impl AiToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Parameters
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::acquire_by_login] 创建。
    ///
    /// # Returns
    ///
    /// 返回一个 [AiToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于当前 [CasToken] 过期导致登录失败，此时会返回 [TokenExpired] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &CasToken,
    ) -> Result<Self, crate::Error<TokenExpired>> {
        let mut current_url = INITIAL_AUTH_URL.to_string();
        let mut all_cookies = cas_token.cookie().to_string();
        let mut cas_authenticated = false;

        // Phase 1: 跟随 302/303 链，到达 maas callback
        for _ in 0..10 {
            let res = client
                .get(&current_url)
                .header(COOKIE, &all_cookies)
                .send()
                .await
                .network_err()?;

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
                current_url = resolve_location(&current_url, location).unexpected_err()?;
            } else if current_url.contains("maas.nscc-cs.cn") {
                break;
            } else if current_url.contains("cas.hnu.edu.cn") && !cas_authenticated {
                // 重定向链中遇到 CAS 登录页，需要带上 CAS cookie 去请求 ticket URL。
                // 注意：不能使用 all_cookies，因为 CAS 返回的 200 登录页面会在
                // Set-Cookie 中写入新的 JSESSIONID，这个未认证 session 会覆盖掉
                // TGT cookie 导致 TokenExpired。应该使用原始的 cas_token cookie。
                let temp_token = cas_token.clone();
                let ticket_url = temp_token.get_ticket_url(&current_url).await?;
                current_url = ticket_url;
                cas_authenticated = true;
            } else {
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
            .ok_or(parse_err(&oauth_text))?;

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
    /// # Parameters
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
mod test {
    use crate::{ai::test::get_ai_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_ai_token() -> TestResult<()> {
        let ai_token = get_ai_token().await?;
        println!("{:#?}", ai_token);
        Ok(())
    }
}
