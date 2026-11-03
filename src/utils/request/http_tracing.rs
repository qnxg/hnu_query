//! HTTP 请求 tracing 中间件。
//!
//! 并在响应返回后发射 `DEBUG` 事件，记录：
//! - 请求方法与完整 URL（事件字段）
//! - 响应状态码（事件字段）
//! - 重定向 `Location` 头（事件字段，仅当存在时记录）
//!
//! 中间件本身不创建 Span，避免一次业务调用产生大量请求级 Span。
use crate::utils::obs;
use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response, header::LOCATION};
use reqwest_middleware::{Middleware, Next};
use std::time::Instant;

pub struct HttpTracing;

fn error_kind(error: &reqwest_middleware::Error) -> &'static str {
    match error {
        reqwest_middleware::Error::Reqwest(error) if error.is_timeout() => "timeout",
        reqwest_middleware::Error::Reqwest(error) if error.is_connect() => "connect",
        reqwest_middleware::Error::Reqwest(_) => "request",
        reqwest_middleware::Error::Middleware(_) => "middleware",
    }
}

#[async_trait]
impl Middleware for HttpTracing {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let method = req.method().clone();
        let url = req.url().clone();
        let timer = Instant::now();
        let result = next.run(req, extensions).await;
        let duration = timer.elapsed().as_millis();
        match &result {
            Ok(res) => {
                let status = res.status();
                match res
                    .headers()
                    .get(LOCATION)
                    .and_then(|value| value.to_str().ok())
                {
                    Some(location) => {
                        obs::debug!(
                            event = "http_response",
                            %method,
                            %url,
                            %status,
                            status_code = status.as_u16(),
                            duration_ms = duration,
                            %location,
                            "response"
                        );
                    }
                    None => {
                        obs::debug!(
                            event = "http_response",
                            %method,
                            %url,
                            %status,
                            status_code = status.as_u16(),
                            duration_ms = duration,
                            "response"
                        );
                    }
                }
            }
            Err(error) => {
                obs::debug!(
                    event = "http_request_failed",
                    %method,
                    %url,
                    duration_ms = duration,
                    error = error_kind(error),
                    "request failed"
                );
            }
        }
        result
    }
}
