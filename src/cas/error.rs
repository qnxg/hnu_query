/// 统一身份认证系统令牌过期
///
/// `status_code` 和 `url` 用于诊断具体原因：
/// - 200：TGT cookie 无效，CAS 返回了登录页面而非重定向
/// - 4xx/5xx：CAS 服务端拒绝了请求
#[derive(Debug, Clone)]
pub struct TokenExpired {
    pub status_code: u16,
    pub url: String,
}

impl TokenExpired {
    pub fn new(status_code: u16, url: impl Into<String>) -> Self {
        Self {
            status_code,
            url: url.into(),
        }
    }
}

impl std::fmt::Display for TokenExpired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "统一身份认证系统令牌过期（HTTP {}，URL: {}）",
            self.status_code, self.url
        )
    }
}

impl std::error::Error for TokenExpired {}
