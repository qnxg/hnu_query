use std::error::Error as StdError;

#[derive(thiserror::Error, Debug)]
pub enum Error<E: StdError> {
    /// 意料之外的错误
    ///
    /// 此类错误被假设为不可能发生或者原因难以诊断。
    /// 详情信息需要使用调试打印观察。
    /// 如果同类问题出现可靠的复现方式，请向开发者反馈问题。
    ///
    /// 目前抛出该错误的地方：
    /// - [reqwest::Response::error_for_status] 失败
    /// - [reqwest::Response::text] 失败
    /// - 在非解析数据部分，期望应该解析到一些数据但是没有解析到，
    ///   比如没有在响应头的 Location 找到 `ticket_url`，
    ///   这个算作 [Error::Unexpected] 而不算 [Error::ParseError]
    #[error(
        "出现了意料之外的错误，详情信息请使用调试打印，\
        如果问题存在可靠的复现方式，请向开发者反馈"
    )]
    Unexpected {
        #[source]
        error: Box<dyn StdError + Send + Sync>,
        file: String,
        line: u32,
        column: u32,
    },
    /// 底层请求错误
    ///
    /// 网络问题，请求超时等来自网络环境的问题，建议稍后重试
    #[error(transparent)]
    NetworkError(reqwest::Error),
    /// 数据解析错误
    ///
    /// 该错误意味着遇到了意料之外的数据格式，当前的库暂时无法解析。
    /// 请向开发者反馈以改进本项目
    #[error("数据解析错误")]
    ParseError {
        error: Option<Box<dyn StdError + Send + Sync>>,
        reason: Option<String>,
        data: String,
    },
    /// 其他错误
    ///
    /// 具体错误请见调用函数的文档
    #[error(transparent)]
    Other(#[from] E),
}

pub fn parse_err<T: StdError>(data: &str) -> Error<T> {
    Error::ParseError {
        error: None,
        reason: None,
        data: data.to_string(),
    }
}

pub fn parse_err_with_reason<T: StdError>(data: &str, reason: &str) -> Error<T> {
    Error::ParseError {
        error: None,
        reason: Some(reason.to_string()),
        data: data.to_string(),
    }
}

pub trait MapUnexpectedErr<T, E>
where
    E: StdError,
{
    fn unexpected_err(self) -> Result<T, Error<E>>;
}

impl<T, E, E0> MapUnexpectedErr<T, E> for Result<T, E0>
where
    E0: Into<Box<dyn StdError + Send + Sync>>,
    E: StdError,
{
    /// 转换某一[Result]中的错误为[Error::Unexpected]
    #[track_caller]
    fn unexpected_err(self) -> Result<T, Error<E>> {
        let loc = std::panic::Location::caller();
        self.map_err(|e| Error::Unexpected {
            error: e.into(),
            file: loc.file().to_string(),
            line: loc.line(),
            column: loc.column(),
        })
    }
}

pub trait MapNetworkErr<T, E>
where
    E: StdError,
{
    fn network_err(self) -> Result<T, Error<E>>;
}

impl<T, E> MapNetworkErr<T, E> for Result<T, reqwest::Error>
where
    E: StdError,
{
    /// 将[reqwest::Error]转换为[Error::NetworkError]
    fn network_err(self) -> Result<T, Error<E>> {
        self.map_err(Error::NetworkError)
    }
}

pub trait MapParseErr<T, E>
where
    E: StdError,
{
    fn parse_err(self, data: &str) -> Result<T, Error<E>>;
    fn parse_err_with_reason(self, data: &str, reason: &str) -> Result<T, Error<E>>;
}

impl<T, E, E0> MapParseErr<T, E> for Result<T, E0>
where
    E0: Into<Box<dyn StdError + Send + Sync>>,
    E: StdError,
{
    fn parse_err(self, data: &str) -> Result<T, Error<E>> {
        self.map_err(|e| Error::ParseError {
            error: Some(e.into()),
            reason: None,
            data: data.to_string(),
        })
    }
    fn parse_err_with_reason(self, data: &str, reason: &str) -> Result<T, Error<E>> {
        self.map_err(|e| Error::ParseError {
            error: Some(e.into()),
            reason: Some(reason.to_string()),
            data: data.to_string(),
        })
    }
}
