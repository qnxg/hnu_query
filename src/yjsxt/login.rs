use crate::{
    cas::{self, login::CasToken},
    error::{MapNetworkErr, MapUnexpectedErr, parse_err},
    utils::client,
};
use reqwest::{StatusCode, header::LOCATION};
use urlencoding::decode;
const YJSXT_FROM_CAS_URL: &str =
    "http://cas.hnu.edu.cn/cas/login?service=http://yjsxt.hnu.edu.cn/gmis/oauthLogin/hndxnew?ywdm=";

/// 研究生系统的令牌
#[derive(Debug, Clone)]
pub struct YjsxtToken {
    id: String,
}

impl YjsxtToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Parameters
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::acquire_by_login] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [YjsxtToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于 [CasToken] 过期导致返回 [cas::error::TokenExpired] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &CasToken,
    ) -> Result<Self, crate::Error<cas::error::TokenExpired>> {
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
            .ok_or_else(|| parse_err(redirection))?
            .to_string();
        let new_url = format!("http://yjsxt.hnu.edu.cn{}", redirection);
        let res = client
            .get(&new_url)
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        // 若是使用本科生账号登录研究生系统时，会被重定向到类似地址
        // http://yjsxt.hnu.edu.cn/gmis/(S(1031xvawyr03t2evudmrbbel))/oauthLogin/hndxn....
        // 按前面的逻辑依然可以截取出/gmis/后面的 id，但是实际上会跳转报错页面
        if let Some(location) = res.headers().get(LOCATION) {
            let location_str = location.to_str().unexpected_err()?;
            if location_str.contains("/home/err") {
                let msg = location_str.split("msg=").nth(1).unwrap_or("");
                let msg = decode(msg).expect("UTF-8");
                return Err(format!("研究生系统报错信息:{msg}")).unexpected_err();
            }
        }
        Ok(Self { id })
    }
    /// 从 id 创建 [YjsxtToken]
    ///
    /// # Parameters
    ///
    /// - `id`: 研究生系统令牌对应的 id，可以通过 [YjsxtToken::id] 获取
    ///
    /// # Returns
    ///
    /// 返回一个 [YjsxtToken] 实例
    ///
    /// # Preconditions
    ///
    /// `id` 参数应该是一个合法的可用作 [YjsxtToken] 的 id，否则会导致未定义行为
    pub fn from_id_unchecked(id: &str) -> Self {
        Self { id: id.to_string() }
    }
    /// 获取当前令牌的 id
    ///
    /// 可用于 [YjsxtToken::from_id_unchecked]
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod test {
    use crate::yjsxt::test::get_yjsxt_token;

    #[tokio::test]
    #[ignore]
    pub async fn test_get_yjsxt_token() -> crate::test::TestResult<()> {
        let yjsxt_token = get_yjsxt_token().await?;
        println!("{:#?}", yjsxt_token);
        Ok(())
    }
}
