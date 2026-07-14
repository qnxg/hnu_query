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
/// 方便接入一些 trace ui 工具进行可视化阅读。
///
/// 导出时会合成一个根 span，并把本轮收集到的所有原先无父 span
/// 挂到该根下、统一到同一个 `traceId`，避免一次测试里多个独立根 span
/// 在 UI 中被拆成多条 trace。
#[cfg(feature = "tracing")]
mod otlp_exporter {
    use opentelemetry::trace::{SpanId, SpanKind, TraceError};
    use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};
    use opentelemetry_sdk::trace::{IdGenerator, RandomIdGenerator};
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

    fn span_kind_code(kind: SpanKind) -> i32 {
        match kind {
            SpanKind::Internal => 1,
            SpanKind::Server => 2,
            SpanKind::Client => 3,
            SpanKind::Producer => 4,
            SpanKind::Consumer => 5,
        }
    }

    /// 合成根 span，并将所有收集到的 span 统一到同一条 trace。
    fn build_otlp_spans(acc: &[SpanData]) -> Vec<serde_json::Value> {
        if acc.is_empty() {
            return Vec::new();
        }

        let id_gen = RandomIdGenerator::default();
        let root_trace_id = id_gen.new_trace_id().to_string();
        let root_span_id = id_gen.new_span_id().to_string();
        let start = acc.iter().map(|s| s.start_time).min().unwrap();
        let end = acc.iter().map(|s| s.end_time).max().unwrap();

        let mut spans = Vec::with_capacity(acc.len() + 1);
        spans.push(json!({
            "traceId": &root_trace_id,
            "spanId": &root_span_id,
            "name": "hnu_query_test",
            "kind": 1,
            "startTimeUnixNano": to_nanos(start).to_string(),
            "endTimeUnixNano": to_nanos(end).to_string(),
            "attributes": [
                {"key": "synthetic", "value": {"boolValue": true}},
            ],
            "events": [],
            "status": {"code": 0},
        }));

        for sd in acc {
            // 原先的根 span 挂到合成根下；已有父子关系的保持 parentSpanId 不变
            let parent_span_id = if sd.parent_span_id == SpanId::INVALID {
                root_span_id.clone()
            } else {
                sd.parent_span_id.to_string()
            };

            let attrs: Vec<_> = sd
                .attributes
                .iter()
                .map(|kv| json!({"key": kv.key.as_str(), "value": any_value(&kv.value)}))
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

            spans.push(json!({
                "traceId": &root_trace_id,
                "spanId": sd.span_context.span_id().to_string(),
                "parentSpanId": parent_span_id,
                "name": sd.name.as_ref(),
                "kind": span_kind_code(sd.span_kind.clone()),
                "startTimeUnixNano": to_nanos(sd.start_time).to_string(),
                "endTimeUnixNano": to_nanos(sd.end_time).to_string(),
                "attributes": attrs,
                "events": events,
                "status": {"code": 0},
            }));
        }

        spans
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
                let json_str = {
                    let acc = accumulated.lock().expect("failed to lock accumulated");
                    let spans = build_otlp_spans(&acc);

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
                std::fs::write(path.as_str(), json_str)
                    .map_err(|e| TraceError::Other(Box::new(e)))?;
                Ok(())
            })
        }
    }
}
