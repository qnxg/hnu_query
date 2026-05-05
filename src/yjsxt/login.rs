use crate::{
    cas::login::{AccountIssue, CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::{client, request::cookie_parser},
};
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};

const YJSXT_FROM_CAS_URL: &str =
    "http://cas.hnu.edu.cn/cas/login?service=http://yjsxt.hnu.edu.cn/gmis/oauthLogin/hndxnew?ywdm=";

/// 研究生系统的令牌
#[derive(Debug, Clone)]
pub struct YjsxtToken {
    headers: HeaderMap,
    id: String,
}

impl YjsxtToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Parameters
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::new] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [YjsxtToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题导致登录失败，此时会返回 [AccountIssue] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &mut CasToken,
    ) -> Result<Self, crate::Error<AccountIssue>> {
        let ticket_url = cas_token.get_ticket_url(YJSXT_FROM_CAS_URL).await?;
        let res = client
            .get(&ticket_url)
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        if res.status() != StatusCode::FOUND {
            return Err(format!(
                "获取研究生系统失败，HTTP代码 {} {}",
                res.status(),
                res.text().await.unwrap_or_default()
            ))
            .unexpected_err();
        }
        let redirection = res
            .headers()
            .get(LOCATION)
            .ok_or("获取研究生跳转路径失败")
            .unexpected_err()?
            .to_str()
            .unexpected_err()?;
        let id = redirection
            .split("/gmis/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .ok_or(parse_err(redirection))?
            .to_string();
        let new_url = format!("http://yjsxt.hnu.edu.cn{}", redirection);
        let cookies: String = cookie_parser(
            client
                .get(&new_url)
                .send()
                .await
                .network_err()?
                .error_for_status()
                .unexpected_err()?
                .headers()
                .get_all(SET_COOKIE),
        )
        .join("; ");
        if cookies.is_empty() {
            return Err("获取研究生系统 cookie 失败").unexpected_err();
        }
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
        Ok(Self { headers, id })
    }
    /// 从 [HeaderMap] 和 id 创建 [YjsxtToken]
    ///
    /// # Parameters
    ///
    /// - `headers`: 一个合法的可用作 [YjsxtToken] 的 [HeaderMap]
    /// - `id`: 研究生系统路径中的 id 标识
    ///
    /// # Returns
    ///
    /// 返回一个 [YjsxtToken] 实例
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [YjsxtToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_unchecked(headers: HeaderMap, id: String) -> Self {
        Self { headers, id }
    }
    /// 获取当前令牌的 [HeaderMap]，可用于 [YjsxtToken::from_unchecked]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    /// 获取当前令牌的 id，可用于 [YjsxtToken::from_unchecked]
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod test {
    use crate::yjsxt::test::get_yjsxt_token;

    #[tokio::test]
    #[ignore]
    pub async fn test_get_yjsxt_token() {
        let yjsxt_token = get_yjsxt_token().await.unwrap();
        println!("{:#?}", yjsxt_token);
    }
}
