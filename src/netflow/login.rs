use crate::{
    cas::{self, login::CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, request::cookie_parser},
};
use reqwest::header::{COOKIE, HeaderMap, SET_COOKIE};

const NETFLOW_URL: &str =
    "http://cas.hnu.edu.cn/application/sso.zf?login=B5712DC2FA281C96E053026B3E0A80A6";

/// 校园网流量系统的令牌
#[derive(Debug, Clone)]
pub struct NetflowToken {
    headers: HeaderMap,
}

impl NetflowToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Arguments
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::acquire_by_login] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [NetflowToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于 [CasToken] 过期导致返回 [cas::error::TokenExpired] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &CasToken,
    ) -> Result<Self, crate::Error<cas::error::TokenExpired>> {
        let (s_ticket, cookies) = cas_token.get_sticket(NETFLOW_URL).await?;
        // 发送请求
        let res = client
            .get("http://ll.hnu.edu.cn/login/validate")
            .header(COOKIE, &cookies)
            .form(&[
                ("s_ticket", s_ticket.as_str()),
                ("login_id", cas_token.stu_id()),
                ("password", ""),
                ("null", ""),
            ])
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        // 获取cookies
        let cookies = cookie_parser(res.headers().get_all(SET_COOKIE));
        // 保留Token，有三个.ASPXAUTH，
        // 只要最后面的一个（这里就不写死是cookies[0]和cookies[3]了）
        if cookies.is_empty() {
            return Err("获取到空的cookies").unexpected_err();
        }
        let mut headers = HeaderMap::new();
        let first = cookies
            .first()
            .ok_or("missing first cookie")
            .unexpected_err()?;
        let last = cookies
            .last()
            .ok_or("missing last cookie")
            .unexpected_err()?;
        let cookies = format!("{first}; {last}");
        headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
        Ok(Self { headers })
    }
    /// 从 [HeaderMap] 创建 [NetflowToken]
    ///
    /// # Arguments
    ///
    /// - `headers`: 一个合法的可用作 [NetflowToken] 的 [HeaderMap]
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [NetflowToken] 的 HeaderMap，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }
    /// 获取当前令牌的 [HeaderMap]，可用于 [NetflowToken::from_headers_unchecked]
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
    use crate::netflow::test::get_netflow_token;

    #[tokio::test]
    #[ignore]
    async fn test_netflow() {
        let netflow_token = get_netflow_token().await;
        println!("{:#?}", netflow_token);
    }
}
