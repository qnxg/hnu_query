# 可观测性

本库内置了对 [`tracing`](https://docs.rs/tracing) 的支持，可以通过 feature flag 开启。开启后，库会在关键路径上发射结构化的 span 和事件，调用方可以自由选择订阅者（`tracing-subscriber` fmt、OpenTelemetry / Jaeger / OTLP 等）来收集和导出这些数据。

在 `Cargo.toml` 中开启 `tracing` feature：

```toml
[dependencies]
hnu_query = { version = "...", features = ["tracing"] }
```

不开启时，库内所有 tracing 代码在编译期即被消除，零运行时开销。

在进行实际请求测试（见 [test.md](./test.md)）时开启 `tracing` feature，会在测试时在标准输出中输出美化后的日志信息。如果在 `.env` 文件中配置了 `TEST_TRACE_OUTPUT=filename` （`filename` 不能是空或是 `stdout`）环境变量，那么会生成一个符合 OpenTelemetry 标准的 JSON 格式的文件。你可以将文件内容放入一些 Trace UI 工具中进行可视化（可以使用 <https://widescope.soumendrak.com/editor/> 这个工具）
