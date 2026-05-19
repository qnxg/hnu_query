#[derive(thiserror::Error, Debug)]
pub enum AiError {
    /// 401 认证失败（API Key 无效或过期）
    #[error("authentication failed: {0}")]
    Authentication(String),
    /// 429 速率限制
    #[error("rate limit exceeded: {0}")]
    RateLimit(String),
    /// 400 请求格式错误
    #[error("bad request: {0}")]
    BadRequest(String),
    /// 402 额度不足
    #[error("insufficient quota: {0}")]
    InsufficientQuota(String),
    /// 422 参数错误
    #[error("invalid parameters: {0}")]
    InvalidParameter(String),
    /// 500 服务器内部错误
    #[error("server error: {0}")]
    ServerError(String),
    /// 503 服务器繁忙
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
    /// 其他 API 错误
    #[error("API error (HTTP {status}): {message}")]
    ApiError { status: u16, message: String },
    /// SSE 流解析错误
    #[error("SSE stream parse error: {0}")]
    StreamParse(String),
}
