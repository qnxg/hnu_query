//! HTTP 请求 tracing 中间件。
//!
//! 为每个经过 [`crate::utils::client`] 的 HTTP 请求创建 `DEBUG` 级 span，
//! 并在响应返回后发射 `DEBUG` 事件，记录：
//! - 请求方法与完整 URL（span 字段，请求开始时即记录）
//! - 上游响应状态、类型、重定向位置和耗时（span 字段）
//! - 传输错误的低基数分类
use crate::utils::obs;
use async_trait::async_trait;
use http::Extensions;
use reqwest::{
    Request, Response,
    header::{CONTENT_TYPE, LOCATION},
};
use reqwest_middleware::{Middleware, Next};
use std::time::Instant;
use tracing::{Instrument, field::Empty};

pub struct HttpTracing;

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
        let host = url.host_str().unwrap_or("unknown").to_owned();
        let timer = Instant::now();
        let span = tracing::debug_span!(
            "upstream.http",
            upstream_name = %host,
            url = %url,
            http_method = %method,
            upstream_http_status_code = Empty,
            upstream_content_type = Empty,
            upstream_duration_ms = Empty,
            upstream_transport_error = Empty,
            redirect_location = Empty,
        );
        let result = next.run(req, extensions).instrument(span.clone()).await;
        let duration = timer.elapsed().as_millis();
        match &result {
            Ok(res) => {
                let status = res.status();
                span.record("upstream_http_status_code", status.as_u16() as i64);
                span.record("upstream_duration_ms", duration as u64);
                if let Some(content_type) = res
                    .headers()
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                {
                    span.record("upstream_content_type", content_type);
                }
                match res.headers().get(LOCATION).and_then(|v| v.to_str().ok()) {
                    Some(location) => {
                        span.record("redirect_location", location);
                        obs::debug!(
                            %method,
                            %url,
                            %status,
                            duration_ms = %duration,
                            location = %location,
                            "upstream response"
                        );
                    }
                    None => {
                        obs::debug!(%method, %url, %status, duration_ms = %duration, "upstream response");
                    }
                }
            }
            Err(e) => {
                let error_kind = match e {
                    reqwest_middleware::Error::Reqwest(error) if error.is_timeout() => "timeout",
                    reqwest_middleware::Error::Reqwest(error) if error.is_connect() => "connect",
                    reqwest_middleware::Error::Reqwest(_) => "request",
                    reqwest_middleware::Error::Middleware(_) => "middleware",
                };
                span.record("upstream_duration_ms", duration as u64);
                span.record("upstream_transport_error", error_kind);
                obs::debug!(
                    %method,
                    %url,
                    duration_ms = %duration,
                    error = ?e,
                    error_kind,
                    "upstream request failed"
                );
            }
        }
        result
    }
}
