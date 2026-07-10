/// CG 系统登录相关的错误
#[derive(thiserror::Error, Debug, Clone)]
pub enum LoginError {
    /// 验证码错误
    #[error("验证码错误")]
    CaptchaError,
    /// 密码错误
    #[error("密码错误")]
    PasswordError,
    /// 未知登录失败
    #[error("登录失败: {0}")]
    LoginFailed(String),
}
