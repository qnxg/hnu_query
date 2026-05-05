#[derive(thiserror::Error, Debug, Clone)]
#[error("研究生系统令牌过期")]
pub struct TokenExpired;
