use crate::{
    cas::login::{AccountIssue, CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, request::cookie_parser},
};
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, SET_COOKIE},
};

// WARN 注意这个url后面必须带`/`，不然无法正常跳转
const PT_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=https://pt.hnu.edu.cn/";

/// 个人门户令牌
#[derive(Debug, Clone)]
pub struct PtToken {
    headers: HeaderMap,
}

impl PtToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Parameters
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::new] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [PtToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题导致登录失败，此时会返回 [AccountIssue] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &mut CasToken,
    ) -> Result<Self, crate::Error<AccountIssue>> {
        let ticket_url = cas_token.get_ticket_url(PT_URL).await?;
        let res = client
            .get(ticket_url)
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        if res.status() != StatusCode::FOUND {
            return Err(format!("登录个人门户失败，HTTP 状态码: {}", res.status()))
                .unexpected_err();
        }
        let cookies = cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ");
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
        Ok(Self { headers })
    }
    /// 从 [HeaderMap] 创建 [PtToken]
    ///
    /// # Arguments
    ///
    /// - `headers`: 一个合法的可用作 [PtToken] 的 [HeaderMap]
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [PtToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }
    /// 获取当前令牌的 [HeaderMap]，可用于 [PtToken::from_headers_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌的 [HeaderMap]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

#[cfg(test)]
mod tests {

    use crate::pt::test::get_pt_token;

    #[tokio::test]
    #[ignore]
    async fn test_pt() {
        let pt_token = get_pt_token().await.unwrap();
        println!("{:#?}", pt_token);
    }
}
