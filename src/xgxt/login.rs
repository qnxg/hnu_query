use crate::{
    cas::login::{AccountIssue, CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, request::cookie_parser},
};
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, SET_COOKIE},
};

const XGXT_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=http://xgxt.hnu.edu.cn/zftal-xgxt-web/teacher/xtgl/index/check.zf";

#[derive(Debug, Clone)]
pub struct XgxtToken {
    headers: HeaderMap,
}

impl XgxtToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Parameters
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::new] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [XgxtToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题导致登录失败，此时会返回 [AccountIssue] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &mut CasToken,
    ) -> Result<Self, crate::Error<AccountIssue>> {
        let ticket_url = cas_token.get_ticket_url(XGXT_URL).await?;
        // cas 下发的 ticket_url 是 http 的，但是学工系统要用 https
        let res = client
            .get(ticket_url.replace("http://", "https://"))
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        if res.status() != StatusCode::FOUND {
            return Err(format!("获取学工系统失败，HTTP代码 {}", res.status())).unexpected_err();
        }
        let cookies: String = cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ");
        if cookies.is_empty() {
            return Err("获取学工系统失败，接收到空的 cookie").unexpected_err();
        }
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
        Ok(Self { headers })
    }
    /// 从 [HeaderMap] 创建 [XgxtToken]
    ///
    /// # Arguments
    ///
    /// - `headers`: 一个合法的可用作 [XgxtToken] 的 [HeaderMap]
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [XgxtToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }
    /// 获取当前令牌的 [HeaderMap]，可用于 [XgxtToken::from_headers_unchecked]
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

    use crate::xgxt::test::get_xgxt_token;

    #[tokio::test]
    #[ignore]
    async fn test_xgxt() {
        let xgxt_token = get_xgxt_token().await.unwrap();
        println!("{:#?}", xgxt_token);
    }
}
