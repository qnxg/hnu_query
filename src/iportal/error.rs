/// iportal 登录会话已失效
#[derive(thiserror::Error, Debug, Clone)]
#[error("iportal 令牌过期")]
pub struct IPortalTokenExpired();
