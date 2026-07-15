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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_error_display() -> crate::test::TestResult<()> {
        assert_eq!(LoginError::CaptchaError.to_string(), "验证码错误");
        assert_eq!(LoginError::PasswordError.to_string(), "密码错误");
        assert_eq!(
            LoginError::LoginFailed("测试原因".into()).to_string(),
            "登录失败: 测试原因"
        );
        Ok(())
    }

    #[test]
    fn test_cg_error_display() -> crate::test::TestResult<()> {
        assert_eq!(CgError::TokenExpired.to_string(), "令牌已过期，请重新登录");
        assert_eq!(CgError::CourseNotFound.to_string(), "未找到课程信息");
        assert_eq!(CgError::AssignmentNotFound.to_string(), "未找到作业信息");
        Ok(())
    }

    #[test]
    fn test_login_error_clone() -> crate::test::TestResult<()> {
        let err = LoginError::LoginFailed("test".into());
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
        Ok(())
    }

    #[test]
    fn test_cg_error_clone() -> crate::test::TestResult<()> {
        let err = CgError::TokenExpired;
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
        Ok(())
    }
}
