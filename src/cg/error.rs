#[derive(thiserror::Error, Debug, Clone)]
#[error("CG 系统令牌过期")]
pub struct TokenExpired;
