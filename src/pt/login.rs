use crate::{
    cas::{self, login::CasToken},
    error::{CheckStatusCodeErr, MapNetworkErr, MapParseErr, MapUnexpectedErr},
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
    /// # Arguments
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::acquire_by_login] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [PtToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于 [CasToken] 过期导致返回 [cas::error::TokenExpired] 错误
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(cas_token), fields(subsystem = "pt"), err)
    )]
    pub async fn acquire_by_cas_login(
        cas_token: &CasToken,
    ) -> Result<Self, crate::Error<cas::error::TokenExpired>> {
        let ticket_url = cas_token.get_ticket_url(PT_URL).await?;
        let res = client
            .get(ticket_url)
            .send()
            .await
            .network_err()?
            .status_code_err()
            .await?;
        let status = res.status();
        if status != StatusCode::FOUND {
            #[cfg(feature = "tracing")]
            {
                use crate::utils::obs;
                let body = res.text().await.unwrap_or_default();
                obs::error!(status = %status, body = %body, "unexpected_status");
            }
            return Err(format!("登录个人门户失败，HTTP 状态码: {}", status)).unexpected_err();
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
        let pt_token = get_pt_token().await;
        println!("{:#?}", pt_token);
    }
}
