use crate::{
    cas::{self, login::CasToken},
    error::{CheckStatusCodeErr, MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, obs, request::cookie_parser},
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
    /// # Arguments
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::acquire_by_login] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [XgxtToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于 [CasToken] 过期导致返回 [cas::error::TokenExpired] 错误
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(cas_token), fields(subsystem = "xgxt"), err)
    )]
    pub async fn acquire_by_cas_login(
        cas_token: &CasToken,
    ) -> Result<Self, crate::Error<cas::error::TokenExpired>> {
        let ticket_url = cas_token.get_ticket_url(XGXT_URL).await?;
        obs::debug!(ticket_url = %ticket_url, "original_ticket_url");
        // cas 下发的 ticket_url 是 http 的，但是学工系统要用 https
        let res = client
            .get(ticket_url.replace("http://", "https://"))
            .send()
            .await
            .network_err()?
            .status_code_err()
            .await?;
        let status = res.status();
        if status != StatusCode::FOUND {
            #[cfg(feature = "tracing")]
            {
                let body = res.text().await.unwrap_or_default();
                obs::error!(status = %status, body = %body, "unexpected_status");
            }
            return Err(format!("获取学工系统失败，HTTP代码 {}", status)).unexpected_err();
        }
        let cookies: String = cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ");
        if cookies.is_empty() {
            #[cfg(feature = "tracing")]
            {
                let body = res.text().await.unwrap_or_default();
                obs::error!(body = %body, "empty_cookie");
            }
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
    use crate::{test::TestResult, xgxt::test::get_xgxt_token};

    #[tokio::test]
    #[ignore]
    async fn test_xgxt() -> TestResult<()> {
        let xgxt_token = get_xgxt_token().await?;
        println!("{:#?}", xgxt_token);
        Ok(())
    }
}
