#[derive(thiserror::Error, Debug, Clone)]
#[error("iportal 令牌过期")]
pub struct IPortalExpired();
