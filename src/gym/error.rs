#[derive(thiserror::Error, Debug, Clone)]
#[error("体测系统令牌过期")]
pub struct TokenExpired;
