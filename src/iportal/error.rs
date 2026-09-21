/// iportal 登录会话已失效
#[derive(thiserror::Error, Debug, Clone)]
#[error("iportal 令牌过期")]
pub struct IPortalTokenExpired;

pub trait CheckIPortalTokenExpired {
    /// 检查错误 iportal 登陆是否失效
    fn iportal_token_expired(self) -> Result<Self, crate::Error<IPortalTokenExpired>>
    where
        Self: Sized;
}

impl CheckIPortalTokenExpired for reqwest::Response {
    fn iportal_token_expired(self) -> Result<Self, crate::Error<IPortalTokenExpired>>
    where
        Self: Sized,
    {
        if self.status() == reqwest::StatusCode::FOUND {
            Err(crate::Error::Other(IPortalTokenExpired))
        } else {
            Ok(self)
        }
    }
}
