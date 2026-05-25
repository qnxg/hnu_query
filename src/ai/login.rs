use crate::{
    cas::login::{AccountIssue, CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::{client, request::cookie_parser},
};
use reqwest::{
    StatusCode,
    header::{AUTHORIZATION, COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};
use serde_json::Value;

// 登录入口，OAuth2 authorize 端点
const INITIAL_AUTH_URL: &str = "https://deepseek.hnu.edu.cn:5556/auth?client_id=openclaw&response_type=code&redirect_uri=https%3A%2F%2Fmaas.nscc-cs.cn%2Fcallback&scope=openid%20profile%20email";
// maas 平台 OAuth2 token 端点，通过 code 得到 bearer token
const OAUTH_LOGIN_URL: &str = "https://maas.nscc-cs.cn/api/oauth-login";

/// AI 系统的令牌
///
/// `headers` 包含 `Authorization: Bearer <token>` 和 `Cookie: session=...`
#[derive(Debug, Clone)]
pub struct AiToken {
    headers: HeaderMap,
}

/// 将相对路径的 Location 解析为绝对 URL
fn resolve_location(current_url: &str, location: &str) -> String {
    if location.starts_with("http://") || location.starts_with("https://") {
        return location.to_string();
    }
    let after_proto = match current_url.find("://") {
        Some(pos) => &current_url[pos + 3..],
        None => return format!("{}{}", current_url.trim_end_matches('/'), location),
    };
    match after_proto.find('/') {
        Some(path_start) => {
            let origin_end = current_url.len() - after_proto.len() + path_start;
            format!("{}{}", &current_url[..origin_end], location)
        }
        None => format!("{}{}", current_url.trim_end_matches('/'), location),
    }
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
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::new] 创建。
    ///
    /// # Returns
    ///
    /// 返回一个 [AiToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题导致登录失败，此时会返回 [AccountIssue] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &CasToken,
    ) -> Result<Self, crate::Error<AccountIssue>> {
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
                current_url = resolve_location(&current_url, location);
            } else if current_url.contains("maas.nscc-cs.cn") {
                break;
            } else if current_url.contains("cas.hnu.edu.cn") && !cas_authenticated {
                // 重定向链中遇到 CAS 登录页，需要带上 CAS cookie + OAuth2
                // session cookie 去请求 ticket URL
                let merged = format!("{}; {}", cas_token.cookie(), all_cookies);
                let temp_token =
                    CasToken::from_cookie_unchecked(&merged, cas_token.stu_id());
                let ticket_url =
                    temp_token.get_ticket_url(&current_url).await.unexpected_err()?;
                current_url = ticket_url;
                cas_authenticated = true;
            } else {
                return Err(format!(
                    "未预期的HTTP状态码 {}，URL: {}",
                    status, current_url
                ))
                .unexpected_err();
            }
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
    use crate::ai::test::get_ai_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_ai_token() {
        let ai_token = get_ai_token().await.unwrap();
        println!("{:#?}", ai_token);
    }
}
