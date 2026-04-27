use crate::cas::login::{AccountIssue, CasToken};
use crate::error::{MapNetworkErr, MapParseErr, MapUnexpectedErr};
use crate::utils::client;
use reqwest::header::HeaderMap;
use serde_json::Value;

const CA_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=https://ca.hnu.edu.cn/student/";

/// 可信电子凭证的令牌
#[derive(Debug, Clone)]
pub struct CaToken {
    headers: HeaderMap,
}

impl CaToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Parameters
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::new] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [CaToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题导致登录失败，此时会返回 [AccountIssue] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &mut CasToken,
    ) -> Result<Self, crate::Error<AccountIssue>> {
        let ticket_url = cas_token.get_ticket_url(CA_URL).await?;
        client
            .get(&ticket_url)
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        // TODO 这里可能会 panic
        let ticket = ticket_url.split("ticket=").collect::<Vec<&str>>()[1];
        let json_str =
        client.get(format!("https://ca.hnu.edu.cn/student/cas/client/validateLogin?ticket={ticket}%23%2F&service=https:%2F%2Fca.hnu.edu.cn%2Fstudent%2F"))
        .send().await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text().await
        .unexpected_err()?;

        let json: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;

        if json["message"] != "登录成功" {
            return Err(format!("登录失败: {}", json["message"])).unexpected_err();
        }
        // TODO 这里可能会 panic？
        let token = json["result"]["token"].as_str().unwrap();
        let cookie = format!("X-Access-Token={token}");
        let mut headers = HeaderMap::new();
        headers.insert("X-Access-Token", token.parse().parse_err(token)?);
        headers.insert("Cookie", cookie.parse().parse_err(&cookie)?);
        Ok(Self { headers })
    }
    /// 从 [HeaderMap] 创建 [CaToken]
    ///
    /// # Parameters
    ///
    /// - `headers`: 一个合法的可用作 [CaToken] 的 [HeaderMap]
    ///
    /// # Returns
    ///
    /// 返回一个 [CaToken] 实例
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [CaToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }
    /// 获取当前令牌的 [HeaderMap]，可用于 [CaToken::from_headers_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌的 [HeaderMap]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

#[cfg(test)]
mod test {
    use crate::ca::test::get_ca_token;

    #[tokio::test]
    #[ignore]
    pub async fn test_get_ca_token() {
        let ca_token = get_ca_token().await.unwrap();
        println!("{:#?}", ca_token);
    }
}
