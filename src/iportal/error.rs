use std::error::Error as StdError;

use crate::Error;

/// iportal 登录会话已失效
#[derive(thiserror::Error, Debug)]
#[error("iportal 令牌过期")]
pub struct IPortalTokenExpired {
    #[source]
    error: Box<dyn StdError + Send + Sync>,
    file: String,
    line: u32,
    column: u32,
}

pub trait CheckIPortalTokenExpired {
    /// 检查错误 iportal 登陆是否失效
    fn iportal_token_expired(self) -> Result<Self, Error<IPortalTokenExpired>>
    where
        Self: Sized;
}

impl CheckIPortalTokenExpired for reqwest::Response {
    #[track_caller]
    fn iportal_token_expired(self) -> Result<Self, Error<IPortalTokenExpired>>
    where
        Self: Sized,
    {
        let loc = std::panic::Location::caller();
        let file = loc.file().to_string();
        let line = loc.line();
        let column = loc.column();
        if self.status() == reqwest::StatusCode::FOUND {
            Err(Error::Other(IPortalTokenExpired {
                error: "iportal 令牌过期".into(),
                file,
                line,
                column,
            }))
        } else {
            Ok(self)
        }
    }
}
