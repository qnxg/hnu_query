#[derive(thiserror::Error, Debug, Clone)]
#[error("教务系统令牌过期")]
pub struct TokenExpired;
