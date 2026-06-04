#[derive(thiserror::Error, Debug, Clone)]
#[error("统一身份认证系统令牌过期")]
pub struct TokenExpired;
