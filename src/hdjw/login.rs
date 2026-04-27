use crate::{
    cas::login::{AccountIssue, CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, request::cookie_parser},
};
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};

const HDJW_FROM_CAS_URL: &str =
    "http://cas.hnu.edu.cn/cas/login?service=http://hdjw.hnu.edu.cn/gld/sso.jsp";
const HDJW_ENTER_URL: &str = "http://hdjw.hnu.edu.cn/gld/sso.jsp";

/// 教务系统的令牌
#[derive(Debug, Clone)]
pub struct HdjwToken {
    headers: HeaderMap,
}

impl HdjwToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Parameters
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::new] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [HdjwToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题导致登录失败，此时会返回 [AccountIssue] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &mut CasToken,
    ) -> Result<Self, crate::Error<AccountIssue>> {
        // 需要先请求 hdjw 的登录页面，获取到相关的 cookie
        let cookies = cookie_parser(
            client
                .get(HDJW_ENTER_URL)
                .send()
                .await
                .network_err()?
                .error_for_status()
                .unexpected_err()?
                .headers()
                .get_all(SET_COOKIE),
        )
        .join("; ");
        let ticket_url = cas_token.get_ticket_url(HDJW_FROM_CAS_URL).await?;
        // 这里需要带着之前拿到的 cookies 去访问 ticket_url，不然会返回 500 internal server
        // error
        client
            .get(ticket_url)
            .header(COOKIE, &cookies)
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        // 上面的请求会重定向到 HDJW_ENTER_URL，我们再访问一下。
        let res = client
            .get(HDJW_ENTER_URL)
            .header(COOKIE, &cookies)
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        // 随后又会被重定向到一个新的链接，再请求一下就会得到 hdjw 鉴权的 cookie
        if res.status() != StatusCode::FOUND {
            return Err(format!(
                "获取教务系统失败，HTTP代码 {} {}",
                res.status(),
                res.text().await.unwrap_or_default()
            ))
            .unexpected_err();
        }
        let target_url = res
            .headers()
            .get(LOCATION)
            .ok_or("获取重定向链接失败")
            .unexpected_err()?
            .to_str()
            .unexpected_err()?;
        let new_cookies = cookie_parser(
            client
                .get(target_url)
                .header(COOKIE, &cookies)
                .send()
                .await
                .network_err()?
                .error_for_status()
                .unexpected_err()?
                .headers()
                .get_all(SET_COOKIE),
        )
        .join("; ");
        // 保险起见，将两次 cookie 合并一下
        let cookies = format!("{}; {}", cookies, new_cookies);
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
        Ok(Self { headers })
    }
    /// 从 [HeaderMap] 创建 [HdjwToken]
    ///
    /// # Parameters
    ///
    /// - `headers`: 一个合法的可用作 [HdjwToken] 的 [HeaderMap]
    ///
    /// # Returns
    ///
    /// 返回一个 [HdjwToken] 实例
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [HdjwToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }
    /// 获取当前令牌的 [HeaderMap]，可用于 [HdjwToken::from_headers_unchecked]
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
    use crate::hdjw::test::get_hdjw_token;

    #[tokio::test]
    #[ignore]
    pub async fn test_get_hdjw_token() {
        let hdjw_token = get_hdjw_token().await.unwrap();
        println!("{:#?}", hdjw_token);
    }
}
