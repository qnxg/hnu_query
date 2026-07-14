//! HTTP 请求 tracing 中间件。
//!
//! 为每个经过 [`crate::utils::client`] 的 HTTP 请求创建 `DEBUG` 级 span，
//! 并在响应返回后发射 `DEBUG` 事件，记录：
//! - 请求方法与完整 URL（span 字段，请求开始时即记录）
//! - 响应状态码（事件字段）
//! - 重定向 `Location` 头（事件字段，仅当存在时记录）
use crate::utils::obs;
use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response, header::LOCATION};
use reqwest_middleware::{Middleware, Next};
use std::time::Instant;

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
        let timer = Instant::now();
        let result = next.run(req, extensions).await;
        let duration = timer.elapsed().as_micros();
        match &result {
            Ok(res) => {
                let status = res.status();
                match res.headers().get(LOCATION).and_then(|v| v.to_str().ok()) {
                    Some(loc) => {
                        obs::debug!(%method, %url, %status, duration = %duration, location = %loc, "response");
                    }
                    None => {
                        obs::debug!(%method, %url, %status, duration = %duration, "response");
                    }
                }
            }
            Err(e) => {
                obs::debug!(%method, %url, duration = %duration, error = ?e, "request failed");
            }
        }
        result
    }
}
