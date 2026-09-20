use crate::{
    cas::{self, login::CasToken},
    error::{CheckStatusCodeErr, MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, request::cookie_parser},
};
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};

/// iportal 令牌
#[derive(Debug, Clone)]
pub struct IPortalToken {
    headers: HeaderMap,
}

const IPORTAL_LOGIN_URL: &str = "https://cas.hnu.edu.cn/cas/login?service=https%3A%2F%2Fiportal.hnu.edu.cn%2Fhnu%2Ffrontend%2Flogin%3Fredirect%3Dhttps%253A%252F%252Fiportal.hnu.edu.cn%252Fhome&isotherLogin=true";

impl IPortalToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Arguments
    ///
    /// - `cas_token`: 统一身份认证系统的令牌, 可以通过 [CasToken::acquire_by_login] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [IPortalToken] 实例
    ///
    /// # Errors
    ///
    /// 当 [`CasToken`] 过期、网络请求失败或登录响应不符合预期时返回错误
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(cas_token), fields(subsystem = "iportal"), err)
    )]
    pub async fn acquire_by_cas_login(
        cas_token: &CasToken,
    ) -> Result<Self, crate::Error<cas::error::TokenExpired>> {
        let ticket_url = cas_token.get_ticket_url(IPORTAL_LOGIN_URL).await?;
        let res = client
            .get(&ticket_url)
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
            return Err(format!("登录 iportal 失败, HTTP 状态码: {status}")).unexpected_err();
        }

        let mut cookies = Vec::new();

        merge_cookies(
            &mut cookies,
            cookie_parser(res.headers().get_all(SET_COOKIE)),
        );

        if let Some(location) = res.headers().get(LOCATION) {
            let location = location.to_str().unexpected_err()?;
            let redirect_request = client.get(location);
            let redirect_request = if cookies.is_empty() {
                redirect_request
            } else {
                redirect_request.header(COOKIE, cookies.join("; "))
            };
            let redirect_response = redirect_request
                .send()
                .await
                .network_err()?
                .status_code_err()
                .await?;
            merge_cookies(
                &mut cookies,
                cookie_parser(redirect_response.headers().get_all(SET_COOKIE)),
            );
        }

        let cookies = cookies.join("; ");
        if cookies.is_empty() {
            return Err("登录iportal失败: 响应中没有 Cookie".to_string()).unexpected_err();
        }
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
        Ok(Self { headers })
    }

    /// 从 [HeaderMap] 创建 [IPortalToken]
    ///
    /// # Arguments
    ///
    /// - `headers`: 一个合法的可用作 [IPortalToken] 的 [HeaderMap]
    ///
    /// # Returns
    ///
    /// 返回一个使用给定请求头的 [`IPortalToken`]
    ///
    /// # Preconditions
    ///
    /// `headers` 应包含当前有效 iportal 会话的 `Cookie` 请求头, 本函数不会验证其有效性
    /// 无效请求头会使后续查询返回错误
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }

    /// 获取当前令牌的 [HeaderMap]，可用于 [IPortalToken::from_headers_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌的 [HeaderMap]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

fn merge_cookies(cookies: &mut Vec<String>, new_cookies: impl IntoIterator<Item = String>) {
    for cookie in new_cookies {
        let name = cookie
            .split_once('=')
            .map_or(cookie.as_str(), |(name, _)| name);
        if let Some(index) = cookies.iter().position(|existing| {
            existing
                .split_once('=')
                .map_or(existing.as_str(), |(existing_name, _)| existing_name)
                == name
        }) {
            cookies[index] = cookie;
        } else {
            cookies.push(cookie);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{iportal::test, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_login() -> TestResult<()> {
        let token = test::get_iportal_token().await?;
        println!("{token:#?}");
        Ok(())
    }
}
