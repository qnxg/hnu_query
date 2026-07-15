/// CG 系统操作相关错误
#[derive(thiserror::Error, Debug, Clone)]
pub enum CgError {
    /// 令牌过期，需要重新登录
    #[error("令牌已过期，请重新登录")]
    TokenExpired,
    /// 未找到课程信息
    #[error("未找到课程信息")]
    CourseNotFound,
    /// 未找到作业信息
    #[error("未找到作业信息")]
    AssignmentNotFound,
}
