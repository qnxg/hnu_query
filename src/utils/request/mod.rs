#[cfg(feature = "tracing")]
mod http_tracing;

use reqwest::{
    Client, Response,
    header::{GetAll, HeaderValue},
    redirect::Policy,
};
use reqwest_middleware::ClientBuilder;
use std::{sync::LazyLock, time::Duration};

#[cfg(feature = "tracing")]
/// Supported response body formats used by the observability layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyFormat {
    Json,
    Html,
    Pdf,
    Text,
    Binary,
    Empty,
}

#[cfg(feature = "tracing")]
impl BodyFormat {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Html => "html",
            Self::Pdf => "pdf",
            Self::Text => "text",
            Self::Binary => "binary",
            Self::Empty => "empty",
        }
    }
}

#[cfg(feature = "tracing")]
fn detect_body_format(body: &[u8]) -> BodyFormat {
    let body = body.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(body);
    let start = body
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(body.len());
    let body = &body[start..];
    let first = body.first().copied();
    match first {
        None => return BodyFormat::Empty,
        Some(b'%') if body.starts_with(b"%PDF") => return BodyFormat::Pdf,
        Some(b'{') | Some(b'[') => return BodyFormat::Json,
        Some(b'<') => return BodyFormat::Html,
        Some(_) => {}
    }
    if std::str::from_utf8(body).is_ok() {
        BodyFormat::Text
    } else {
        BodyFormat::Binary
    }
}

/// Reads response bodies while recording non-sensitive format metadata.
pub trait ObservedResponseExt {
    /// Reads a response as text and records its detected format when tracing is enabled.
    async fn observed_text(self) -> Result<String, reqwest::Error>;

    /// Reads a response as bytes and records its detected format when tracing is enabled.
    async fn observed_bytes(self) -> Result<bytes::Bytes, reqwest::Error>;
}

impl ObservedResponseExt for Response {
    async fn observed_text(self) -> Result<String, reqwest::Error> {
        #[cfg(not(feature = "tracing"))]
        {
            return self.text().await;
        }

        #[cfg(feature = "tracing")]
        {
            let content_type = self
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            let body = self.text().await?;
            let format = detect_body_format(body.as_bytes());
            crate::utils::obs::record_body_metadata(content_type.as_deref(), format.as_str());
            Ok(body)
        }
    }

    async fn observed_bytes(self) -> Result<bytes::Bytes, reqwest::Error> {
        #[cfg(not(feature = "tracing"))]
        {
            return self.bytes().await;
        }

        #[cfg(feature = "tracing")]
        {
            let content_type = self
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            let body = self.bytes().await?;
            let format = detect_body_format(&body);
            crate::utils::obs::record_body_metadata(content_type.as_deref(), format.as_str());
            Ok(body)
        }
    }
}

pub static CLIENT: LazyLock<reqwest_middleware::ClientWithMiddleware> = LazyLock::new(|| {
    let inner = Client::builder()
        // 需要忽略无效证书，因为图书馆的https证书是寄的，而采用http请求的话cas下发的ticket不被图书系统接受
        .danger_accept_invalid_certs(true)
        .connection_verbose(false)
        // 设置超时是重要的，避免超时中间件触发后任务仍在进行
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(2))
        .pool_idle_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(2000)    // 部署到生产环境一定要适当调整，Linux系统默认TCP上限是1024，本程序大致访问6个host左右，设置一个合理值不要超过上限，如系统没有网络调优情况下可设置为100
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")  // 必须加才会通过cas的验证
        // .user_agent("reqwest/0.12.8")
        .no_proxy() // 禁止代理，防止调试时候系统代理的影响
        .redirect(Policy::none()) // 禁止自动重定向方便操作，目前有几个接口依赖于禁止重定向，因此不能直接允许重定向
        // .http1_title_case_headers()
        .build()
        .expect("构建client失败");

    let builder = ClientBuilder::new(inner);
    #[cfg(feature = "tracing")]
    let builder = builder.with(http_tracing::HttpTracing);
    builder.build()
});

/// 项目的cookie_parser，旨在只保留key=value的字符串形式
#[inline]
pub fn cookie_parser(cookie: GetAll<HeaderValue>) -> Vec<String> {
    cookie
        .iter()
        .filter_map(cookie_parser_inner)
        .collect::<Vec<String>>()
}

#[inline]
fn cookie_parser_inner(cookie: &HeaderValue) -> Option<String> {
    let cookie = cookie
        .to_str()
        .expect("异常cookie")
        .split(';')
        .collect::<Vec<&str>>();
    let pair: Vec<&str> = cookie[0].split('=').collect();
    if pair[1].is_empty() {
        return None;
    }
    Some(format!("{}={}", pair[0], pair[1]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[tokio::test]
    #[ignore]
    async fn test_init_request() -> TestResult<()> {
        let res = CLIENT.get("https://www.baidu.com").send().await?;
        assert!(res.status().is_success());
        Ok(())
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn detects_supported_response_formats() {
        assert_eq!(detect_body_format(b" \n{\"ok\":true}"), BodyFormat::Json);
        assert_eq!(detect_body_format(b"<html></html>"), BodyFormat::Html);
        assert_eq!(detect_body_format(b"%PDF-1.7"), BodyFormat::Pdf);
        assert_eq!(detect_body_format(b"plain text"), BodyFormat::Text);
        assert_eq!(detect_body_format(&[0xff, 0xfe]), BodyFormat::Binary);
        assert_eq!(detect_body_format(b" \n"), BodyFormat::Empty);
    }
}
