use crate::utils::obs;
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
    /// - [reqwest::Response::status] 返回的代码为 4xx 或是 5xx
    ///   （等价于 [reqwest::Response::error_for_status] 失败）
    /// - [reqwest::Response::text] 失败
    /// - 在非解析数据部分，期望应该解析到一些数据但是没有解析到，
    ///   比如没有在响应头的 Location 找到 `ticket_url`，
    ///   这个算作 [Error::Unexpected] 而不算 [Error::Parse]
    #[error(transparent)]
    Unexpected(UnexpectedError),
    /// 底层请求错误
    ///
    /// 网络问题，请求超时等来自网络环境的问题，建议稍后重试
    #[error("网络错误: {0}")]
    Network(#[source] reqwest::Error),
    /// 数据解析错误
    ///
    /// 该错误意味着遇到了意料之外的数据格式，当前的库暂时无法解析。
    /// 请向开发者反馈以改进本项目
    #[error(transparent)]
    Parse(ParseError),
    /// 其他错误
    ///
    /// 具体错误请见调用函数的文档
    #[error(transparent)]
    Other(E),
}

/// 见 [Error::Unexpected]
#[derive(thiserror::Error, Debug)]
#[error("意料之外的错误：{error}")]
pub struct UnexpectedError {
    #[source]
    error: Box<dyn StdError + Send + Sync>,
    file: String,
    line: u32,
    column: u32,
}

impl UnexpectedError {
    /// 抛出错误的代码文件路径
    pub fn file(&self) -> &str {
        &self.file
    }
    /// 抛出错误的代码行号
    pub fn line(&self) -> u32 {
        self.line
    }
    /// 抛出错误的代码列号
    pub fn column(&self) -> u32 {
        self.column
    }
}

pub trait MapUnexpectedErr<T, E>
where
    E: StdError,
{
    /// 转换某一[Result]中的错误为[Error::Unexpected]
    fn unexpected_err(self) -> Result<T, Error<E>>;
}

impl<T, E, E0> MapUnexpectedErr<T, E> for Result<T, E0>
where
    E0: Into<Box<dyn StdError + Send + Sync>>,
    E: StdError,
{
    #[track_caller]
    fn unexpected_err(self) -> Result<T, Error<E>> {
        let loc = std::panic::Location::caller();
        self.map_err(|e| {
            Error::Unexpected(UnexpectedError {
                error: e.into(),
                file: loc.file().to_string(),
                line: loc.line(),
                column: loc.column(),
            })
        })
    }
}

pub trait MapNetworkErr<T, E>
where
    E: StdError,
{
    /// 将[Result]中的错误转换为[Error::Network]
    fn network_err(self) -> Result<T, Error<E>>;
}

impl<T, E> MapNetworkErr<T, E> for Result<T, reqwest_middleware::Error>
where
    E: StdError,
{
    /// 将 [reqwest_middleware::Error] 转换为 [Error]
    ///
    /// - `Reqwest(reqwest::Error)` 变体映射为 [Error::Network]
    /// - `Middleware(...)` 变体映射为 [Error::Unexpected]
    ///   （middleware 自身出错的概率极低）
    #[track_caller]
    fn network_err(self) -> Result<T, Error<E>> {
        let loc = std::panic::Location::caller();
        self.map_err(|e| match e {
            reqwest_middleware::Error::Reqwest(r) => Error::Network(r),
            reqwest_middleware::Error::Middleware(m) => Error::Unexpected(UnexpectedError {
                error: m.into(),
                file: loc.file().to_string(),
                line: loc.line(),
                column: loc.column(),
            }),
        })
    }
}

/// 见 [Error::Parse]
#[derive(thiserror::Error, Debug)]
#[error("数据解析错误：{error}")]
pub struct ParseError {
    #[source]
    error: Box<dyn StdError + Send + Sync>,
    file: String,
    line: u32,
    column: u32,
    data: String,
}

impl ParseError {
    /// 抛出错误的代码文件路径
    pub fn file(&self) -> &str {
        &self.file
    }
    /// 抛出错误的代码行号
    pub fn line(&self) -> u32 {
        self.line
    }
    /// 抛出错误的代码列号
    pub fn column(&self) -> u32 {
        self.column
    }
    /// 解析失败的数据
    pub fn data(&self) -> &str {
        &self.data
    }
}

#[track_caller]
pub fn parse_err<E: StdError>(reason: &str, data: &str) -> Error<E> {
    let loc = std::panic::Location::caller();
    Error::Parse(ParseError {
        error: reason.into(),
        file: loc.file().to_string(),
        line: loc.line(),
        column: loc.column(),
        data: data.to_string(),
    })
}

pub trait MapParseErr<T, E>
where
    E: StdError,
{
    /// 将[Result]中的错误转换为[Error::Parse]
    ///
    /// `data` 为解析失败的数据
    fn parse_err(self, data: &str) -> Result<T, Error<E>>;
}

impl<T, E, E0> MapParseErr<T, E> for Result<T, E0>
where
    E0: Into<Box<dyn StdError + Send + Sync>>,
    E: StdError,
{
    #[track_caller]
    fn parse_err(self, data: &str) -> Result<T, Error<E>> {
        let loc = std::panic::Location::caller();
        self.map_err(|e| {
            Error::Parse(ParseError {
                error: e.into(),
                file: loc.file().to_string(),
                line: loc.line(),
                column: loc.column(),
                data: data.to_string(),
            })
        })
    }
}

pub trait CheckStatusCodeErr {
    /// 检查响应状态码是否为错误状态码（4xx 或是 5xx），如果是则抛出 [Error::Unexpected]
    ///
    /// 如果开启了 tracing feature，还会将响应体打印到错误日志中
    fn status_code_err<T: StdError>(self) -> impl Future<Output = Result<Self, Error<T>>>
    where
        Self: Sized;
}

impl CheckStatusCodeErr for reqwest::Response {
    // track_caller 在目前 rust 的稳定版中，用在 async fn 上是空操作，所以我们需要先放到同步函数上
    // 然后再返回 future
    #[track_caller]
    fn status_code_err<T: StdError>(self) -> impl Future<Output = Result<Self, Error<T>>>
    where
        Self: Sized,
    {
        let loc = std::panic::Location::caller();
        let file = loc.file().to_string();
        let line = loc.line();
        let column = loc.column();
        async move {
            let status = self.status();
            if status.is_client_error() || status.is_server_error() {
                obs::error!(status = %status, body = %self.text().await.unwrap_or_default(), "status code error");
                Err(Error::Unexpected(UnexpectedError {
                    error: format!("status code error: {status}").into(),
                    file,
                    line,
                    column,
                }))
            } else {
                Ok(self)
            }
        }
    }
}
