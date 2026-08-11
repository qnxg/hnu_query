/// CG 系统操作相关错误
#[derive(thiserror::Error, Debug, Clone)]
pub enum CgError {
    /// 令牌过期，需要重新登录
    #[error("令牌已过期，请重新登录")]
    TokenExpired,
}
