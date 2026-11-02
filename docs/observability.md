# 可观测性

本库内置了对 [`tracing`](https://docs.rs/tracing) 的支持，可以通过 feature flag 开启。开启后，库会在关键路径上发射结构化的 span 和事件，调用方可以自由选择订阅者（`tracing-subscriber` fmt、OpenTelemetry / Jaeger / OTLP 等）来收集和导出这些数据。

在 `Cargo.toml` 中开启 `tracing` feature：

```toml
[dependencies]
hnu_query = { version = "...", features = ["tracing"] }
```

不开启时，库内所有 tracing 代码在编译期即被消除，零运行时开销。

在进行实际请求测试（见 [test.md](./test.md)）时开启 `tracing` feature，会在测试时在标准输出中输出美化后的日志信息。如果在 `.env` 文件中配置了 `TEST_TRACE_OUTPUT=filename` （`filename` 不能是空或是 `stdout`）环境变量，那么会生成一个符合 OpenTelemetry 标准的 JSON 格式的文件。你可以将文件内容放入一些 Trace UI 工具中进行可视化（可以使用 <https://widescope.soumendrak.com/editor/> 这个工具）

## 失败归因字段

开启 tracing 后，业务 span 会声明以下低基数字段：

- `failure_origin`：`upstream_transport`、`upstream_http`、`parser`、`internal` 等归因
- `failure_stage`：`request`、`response`、`parse` 等阶段
- `failure_code`：`timeout`、`http_4xx`、`http_5xx`、`parse_error` 等稳定错误码
- `upstream_http_status_code`：学校系统或其他外部系统真实返回的 HTTP 状态码
- `upstream_outcome`：外部请求成功或 HTTP 错误
- `upstream_body_format`：`json`、`html`、`pdf`、`text`、`binary` 或 `empty`
- `parser_expected_format` / `parser_actual_format`：解析器契约和实际响应格式

`backend` 对客户端返回的状态码属于调用方服务的观测范围，不应与
`upstream_http_status_code` 混用。HTTP middleware 会为每次外部请求创建
`upstream.http` 子 span，记录原始 URL、状态、响应类型、重定向位置和耗时；
响应正文和 `body_bytes` 不写入 trace 或指标。
