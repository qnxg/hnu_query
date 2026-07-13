/// 初始化 tracing 订阅者。
///
/// 仅在 `tracing` feature 开启时生效；feature 关闭时为 no-op。
/// 使用 [`OnceLock`] 保证同进程内多次调用只初始化一次。
#[cfg(feature = "tracing")]
pub fn init_tracing() {
    use std::sync::OnceLock;
    static TRACING_INIT: OnceLock<()> = OnceLock::new();
    TRACING_INIT.get_or_init(|| {
        let _ = dotenvy::dotenv();

        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));

        let output = std::env::var("TEST_TRACE_OUTPUT").unwrap_or_default();
        match output.as_str() {
            "" | "stdout" => {
                tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .with_target(false)
                    .with_level(true)
                    .with_ansi(true)
                    .init();
            }
            path => {
                use opentelemetry::trace::TracerProvider as _;
                use tracing_subscriber::layer::SubscriberExt;
                use tracing_subscriber::util::SubscriberInitExt;

                let exporter = match otlp_exporter::OtlpJsonFileExporter::new(path) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("failed to open trace file {path}: {e}, fallback to stdout");
                        tracing_subscriber::fmt()
                            .with_env_filter(filter)
                            .with_target(false)
                            .with_ansi(true)
                            .init();
                        return;
                    }
                };

                let provider = opentelemetry_sdk::trace::TracerProvider::builder()
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", "hnu_query"),
                    ]))
                    .with_simple_exporter(exporter)
                    .build();
                let tracer = provider.tracer("hnu_query");
                opentelemetry::global::set_tracer_provider(provider);

                tracing_subscriber::registry()
                    .with(filter)
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_target(false)
                            .with_ansi(true),
                    )
                    .with(tracing_opentelemetry::layer().with_tracer(tracer))
                    .init();
            }
        }
    });
}

#[cfg(not(feature = "tracing"))]
pub fn init_tracing() {}

/// 将 tracing-opentelemetry 桥接的 SpanData 序列化为 OTLP/JSON 格式写入文件，
/// 方便接入一些 trace ui 工具进行可视化阅读
#[cfg(feature = "tracing")]
mod otlp_exporter {
    use opentelemetry::trace::{SpanKind, TraceError};
    use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct OtlpJsonFileExporter {
        path: Arc<String>,
        accumulated: Arc<Mutex<Vec<SpanData>>>,
    }

    impl std::fmt::Debug for OtlpJsonFileExporter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("OtlpJsonFileExporter").finish()
        }
    }

    impl OtlpJsonFileExporter {
        pub fn new(path: &str) -> std::io::Result<Self> {
            Ok(Self {
                path: Arc::new(path.to_string()),
                accumulated: Arc::new(Mutex::new(Vec::new())),
            })
        }
    }

    fn to_nanos(t: SystemTime) -> u128 {
        t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos())
    }

    fn any_value(v: &opentelemetry::Value) -> serde_json::Value {
        use opentelemetry::Value;
        match v {
            Value::String(s) => json!({"stringValue": s.as_str()}),
            Value::I64(n) => json!({"intValue": n}),
            Value::F64(f) => json!({"doubleValue": f}),
            Value::Bool(b) => json!({"boolValue": b}),
            _ => json!({"stringValue": v.to_string()}),
        }
    }

    impl SpanExporter for OtlpJsonFileExporter {
        fn export(
            &mut self,
            batch: Vec<SpanData>,
        ) -> futures_util::future::BoxFuture<'static, ExportResult> {
            let path = self.path.clone();
            let accumulated = self.accumulated.clone();
            Box::pin(async move {
                {
                    let mut acc = accumulated.lock().expect("failed to lock accumulated");
                    acc.extend(batch);
                }
                // 重写整个文件为单个完整 JSON
                let accumulated2 = accumulated.clone();
                let path2 = path.clone();
                let json_str = {
                    let acc = accumulated2.lock().expect("failed to lock accumulated");
                    let spans: Vec<_> = acc
                        .iter()
                        .map(|sd| {
                            let parent = if sd.parent_span_id
                                == opentelemetry::trace::SpanId::INVALID
                            {
                                serde_json::Value::Null
                            } else {
                                json!(sd.parent_span_id.to_string())
                            };
                            let kind = match sd.span_kind {
                                SpanKind::Internal => 1,
                                SpanKind::Server => 2,
                                SpanKind::Client => 3,
                                SpanKind::Producer => 4,
                                SpanKind::Consumer => 5,
                            };
                            let attrs: Vec<_> = sd
                                .attributes
                                .iter()
                                .map(|kv| {
                                    json!({"key": kv.key.as_str(), "value": any_value(&kv.value)})
                                })
                                .collect();

                            let events: Vec<_> = sd
                                .events
                                .iter()
                                .map(|ev| {
                                    let ev_attrs: Vec<_> = ev
                                        .attributes
                                        .iter()
                                        .map(|kv| {
                                            json!({"key": kv.key.as_str(), "value": any_value(&kv.value)})
                                        })
                                        .collect();
                                    json!({
                                        "timeUnixNano": to_nanos(ev.timestamp).to_string(),
                                        "name": ev.name,
                                        "attributes": ev_attrs,
                                    })
                                })
                                .collect();

                            let mut span = json!({
                                "traceId": sd.span_context.trace_id().to_string(),
                                "spanId": sd.span_context.span_id().to_string(),
                                "name": sd.name,
                                "kind": kind,
                                "startTimeUnixNano": to_nanos(sd.start_time).to_string(),
                                "endTimeUnixNano": to_nanos(sd.end_time).to_string(),
                                "attributes": attrs,
                                "events": events,
                                "status": {"code": 0},
                            });
                            if !parent.is_null() {
                                span["parentSpanId"] = parent;
                            }
                            span
                        })
                        .collect();

                    let request = json!({
                        "resourceSpans": [{
                            "resource": {
                                "attributes": [
                                    {"key": "service.name", "value": {"stringValue": "hnu_query"}}
                                ]
                            },
                            "scopeSpans": [{
                                "scope": {"name": "hnu_query"},
                                "spans": spans,
                            }]
                        }]
                    });

                    serde_json::to_string_pretty(&request).unwrap_or_default()
                };
                let _ = path2; // path used via closure capture
                std::fs::write(path.as_str(), json_str)
                    .map_err(|e| TraceError::Other(Box::new(e)))?;
                Ok(())
            })
        }
    }
}
