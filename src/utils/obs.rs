//! 可观测性相关的辅助模块，该模块包装了 [tracing] 的一些操作，
//! 当 `tracing` feature 开启时，本模块的宏和函数转发到 [tracing]；
//! 关闭时全部为 no-op，编译期即被完全消除。
#![allow(unused_macros, unused_imports)]

pub use hnu_query_macros::traced;

use std::future::Future;

macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        ::tracing::debug!($($arg)*);
    };
}

macro_rules! info {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        ::tracing::info!($($arg)*);
    };
}

// 宏名不能取 `warn`，否则与 Rust 内置 lint-level 属性 `#[warn(...)]` 冲突（E0659），
macro_rules! warn_evt {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        ::tracing::warn!($($arg)*);
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        ::tracing::error!($($arg)*);
    };
}

macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        ::tracing::trace!($($arg)*);
    };
}

// span 宏：返回一个 entered span guard。
// 调用方应以 `let _guard = obs::debug_span!(...);` 的形式绑定生命周期，
// guard 在 drop 时自动退出 span。feature 关闭时返回 [`NoopSpanGuard`]（非 `()`，
// 避免 `clippy::let_unit_value` 警告）。
macro_rules! debug_span {
    ($name:literal $(, $($field:tt)*)?) => {{
        #[cfg(feature = "tracing")]
        {
            ::tracing::debug_span!($name $(, $($field)*)?).entered()
        }
        #[cfg(not(feature = "tracing"))]
        $crate::utils::obs::NoopSpanGuard
    }};
}

macro_rules! info_span {
    ($name:literal $(, $($field:tt)*)?) => {{
        #[cfg(feature = "tracing")]
        {
            ::tracing::info_span!($name $(, $($field)*)?).entered()
        }
        #[cfg(not(feature = "tracing"))]
        $crate::utils::obs::NoopSpanGuard
    }};
}

/// 创建一个未 enter 的 DEBUG span，并将其与一个 future 关联。主要用于 `try_join!` 等并发场景。
///
/// tracing feature 关闭时，原样返回 future。
///
/// ```ignore
/// obs::instrument!("name", some_future)
/// ```
macro_rules! instrument {
    ($name:literal, $fut:expr $(, $($field:tt)*)?) => {{
        #[cfg(feature = "tracing")]
        {
            use ::tracing::Instrument;
            ($fut).instrument(::tracing::debug_span!($name $(, $($field)*)?))
        }
        #[cfg(not(feature = "tracing"))]
        {
            $fut
        }
    }};
}

/// 向当前 span 回填字段。
///
/// 字段必须在创建该 span 时用 `Empty`（或已有初值）声明过，否则多数订阅者会忽略。
/// tracing feature 关闭时为 no-op。
///
/// ```ignore
/// obs::record!(outcome = "success", count = result.len());
/// ```
macro_rules! record {
    ($($key:ident = $val:expr),+ $(,)?) => {
        #[cfg(feature = "tracing")]
        {
            let __span = ::tracing::Span::current();
            $(
                __span.record(stringify!($key), $val);
            )+
        }
    };
}

/// 将表达式的耗时累加到当前 span 的 `fetch_ms` 属性上。
///
/// 须在 [`traced`] 函数内使用；可配合 `?`
///
/// ```ignore
/// fetch_time!(foo().await)?
/// ```
macro_rules! fetch_time {
    ($expr:expr) => {{
        #[cfg(feature = "tracing")]
        {
            let __obs_span = $crate::utils::obs::phase_span($crate::utils::obs::Phase::Fetch);
            let mut __obs_guard =
                $crate::utils::obs::PhaseTimerGuard::new($crate::utils::obs::Phase::Fetch);
            let __obs_result = {
                use ::tracing::Instrument;
                (async { $expr }).instrument(__obs_span.clone()).await
            };
            __obs_guard.finish($crate::utils::obs::phase_value_succeeded(&__obs_result));
            __obs_result
        }
        #[cfg(not(feature = "tracing"))]
        {
            $expr
        }
    }};
}

/// 将表达式的耗时累加到当前 span 的 `parse_ms` 属性上。
///
/// 须在 [`traced`] 函数内使用；可配合 `?`
///
/// ```ignore
/// parse_time!(parse::foo(&s))?
/// ```
macro_rules! parse_time {
    ($expr:expr) => {{
        #[cfg(feature = "tracing")]
        {
            let __obs_span = $crate::utils::obs::phase_span($crate::utils::obs::Phase::Parse);
            let mut __obs_guard =
                $crate::utils::obs::PhaseTimerGuard::new($crate::utils::obs::Phase::Parse);
            let __obs_result = {
                use ::tracing::Instrument;
                (async { $expr }).instrument(__obs_span.clone()).await
            };
            __obs_guard.finish($crate::utils::obs::phase_value_succeeded(&__obs_result));
            __obs_result
        }
        #[cfg(not(feature = "tracing"))]
        {
            $expr
        }
    }};
}

pub(crate) use {
    debug, debug_span, error, fetch_time, info, info_span, instrument, parse_time, record, trace,
    warn_evt as warning,
};

#[cfg(feature = "tracing")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ResultState {
    #[default]
    Pending,
    Success,
    Error,
}

#[cfg(feature = "tracing")]
impl ResultState {
    fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Pending => None,
            Self::Success => Some("success"),
            Self::Error => Some("error"),
        }
    }
}

#[cfg(feature = "tracing")]
#[derive(Clone, Copy, Debug)]
pub enum Phase {
    Fetch,
    Parse,
}

#[cfg(feature = "tracing")]
struct PhaseState {
    fetch_span: Option<tracing::Span>,
    parse_span: Option<tracing::Span>,
    fetch_ms: u64,
    parse_ms: u64,
    fetch_result: ResultState,
    parse_result: ResultState,
}

#[cfg(feature = "tracing")]
impl Default for PhaseState {
    fn default() -> Self {
        Self {
            fetch_span: None,
            parse_span: None,
            fetch_ms: 0,
            parse_ms: 0,
            fetch_result: ResultState::Pending,
            parse_result: ResultState::Pending,
        }
    }
}

#[cfg(feature = "tracing")]
tokio::task_local! {
    static PHASE_TIMERS: std::cell::RefCell<PhaseState>;
}

/// 由 [`traced`] 注入：在函数体作用域内启用阶段计时器。
pub async fn with_phase_timers<F: Future>(f: F) -> F::Output {
    #[cfg(feature = "tracing")]
    {
        PHASE_TIMERS
            .scope(std::cell::RefCell::new(PhaseState::default()), f)
            .await
    }
    #[cfg(not(feature = "tracing"))]
    {
        f.await
    }
}

/// 由 [`traced`] 注入：函数结束（含 `?` 提前返回）时把累计耗时写入当前 span。
pub struct FlushPhaseTimersOnDrop;

impl Drop for FlushPhaseTimersOnDrop {
    fn drop(&mut self) {
        #[cfg(feature = "tracing")]
        flush_phase_timers_to_current_span();
    }
}

#[cfg(feature = "tracing")]
fn flush_phase_timers_to_current_span() {
    let Ok(state) = PHASE_TIMERS.try_with(|c| {
        let c = c.borrow();
        (
            c.fetch_ms,
            c.parse_ms,
            c.fetch_span.clone(),
            c.parse_span.clone(),
            c.fetch_result,
            c.parse_result,
        )
    }) else {
        return;
    };

    let span = tracing::Span::current();
    span.record("fetch_ms", state.0);
    span.record("parse_ms", state.1);

    if let (Some(fetch_span), Some(result)) = (state.2, state.4.as_str()) {
        fetch_span.record("result", result);
    }
    if let (Some(parse_span), Some(result)) = (state.3, state.5.as_str()) {
        parse_span.record("result", result);
    }
}

#[cfg(feature = "tracing")]
pub fn phase_span(phase: Phase) -> tracing::Span {
    PHASE_TIMERS
        .try_with(|cell| {
            let mut state = cell.borrow_mut();
            match phase {
                Phase::Fetch => state
                    .fetch_span
                    .get_or_insert_with(|| {
                        tracing::debug_span!(
                            "fetch",
                            phase = "fetch",
                            result = tracing::field::Empty,
                        )
                    })
                    .clone(),
                Phase::Parse => state
                    .parse_span
                    .get_or_insert_with(|| {
                        tracing::debug_span!(
                            "parse",
                            phase = "parse",
                            result = tracing::field::Empty,
                        )
                    })
                    .clone(),
            }
        })
        .unwrap_or_else(|_| tracing::Span::none())
}

#[cfg(feature = "tracing")]
pub fn record_phase_result(phase: Phase, success: bool) {
    let _ = PHASE_TIMERS.try_with(|cell| {
        let mut state = cell.borrow_mut();
        let result = match phase {
            Phase::Fetch => &mut state.fetch_result,
            Phase::Parse => &mut state.parse_result,
        };

        if !success {
            *result = ResultState::Error;
        } else if *result != ResultState::Error {
            *result = ResultState::Success;
        }
    });
}

#[cfg(feature = "tracing")]
pub struct PhaseTimerGuard {
    phase: Phase,
    started_at: std::time::Instant,
    finished: bool,
}

#[cfg(feature = "tracing")]
impl PhaseTimerGuard {
    pub fn new(phase: Phase) -> Self {
        Self {
            phase,
            started_at: std::time::Instant::now(),
            finished: false,
        }
    }

    pub fn finish(&mut self, success: bool) {
        if self.finished {
            return;
        }
        self.finished = true;
        add_phase_ms(self.phase, self.started_at.elapsed().as_millis() as u64);
        record_phase_result(self.phase, success);
    }
}

#[cfg(feature = "tracing")]
impl Drop for PhaseTimerGuard {
    fn drop(&mut self) {
        if !self.finished {
            add_phase_ms(self.phase, self.started_at.elapsed().as_millis() as u64);
            record_phase_result(self.phase, false);
        }
    }
}

#[cfg(feature = "tracing")]
pub fn phase_value_succeeded<T: std::fmt::Debug>(value: &T) -> bool {
    let debug = format!("{value:?}");
    !debug.starts_with("Err(") && debug != "None"
}

#[cfg(feature = "tracing")]
fn add_phase_ms(phase: Phase, ms: u64) {
    match phase {
        Phase::Fetch => add_fetch_ms(ms),
        Phase::Parse => add_parse_ms(ms),
    }
}

#[cfg(feature = "tracing")]
pub fn add_fetch_ms(ms: u64) {
    let _ = PHASE_TIMERS.try_with(|c| {
        let mut c = c.borrow_mut();
        c.fetch_ms = c.fetch_ms.saturating_add(ms);
    });
}

#[cfg(feature = "tracing")]
pub fn add_parse_ms(ms: u64) {
    let _ = PHASE_TIMERS.try_with(|c| {
        let mut c = c.borrow_mut();
        c.parse_ms = c.parse_ms.saturating_add(ms);
    });
}

/// 回填业务 Span 的最终结果。
#[cfg(feature = "tracing")]
pub fn record_business_result<T, E>(result: &Result<T, E>) {
    tracing::Span::current().record("result", if result.is_ok() { "success" } else { "error" });
}

#[cfg(not(feature = "tracing"))]
pub fn record_business_result<T, E>(_result: &Result<T, E>) {}

/// feature 关闭时 span 宏返回的占位 guard。
///
/// 存在的唯一作用是让 `let _guard = obs::debug_span!(...)` 在 feature 关闭时
/// 绑定到一个非 `()` 类型，避免 `clippy::let_unit_value` 警告。
#[cfg(not(feature = "tracing"))]
#[expect(unused)]
pub struct NoopSpanGuard;

// 实现 `Drop` 是为了让 `drop(_s)` 在 feature 关闭时也能通过 clippy 的
// `drop_non_drop` 检查（与 `EnteredSpan` 的 API 保持一致）。
#[cfg(not(feature = "tracing"))]
impl Drop for NoopSpanGuard {
    fn drop(&mut self) {}
}

#[cfg(all(test, feature = "tracing"))]
mod tests {
    use super::{fetch_time, parse_time, traced};
    use std::{
        collections::HashMap,
        fmt::Debug,
        sync::{Arc, Mutex},
    };
    use tracing::Subscriber;
    use tracing_subscriber::{Layer, layer::Context, layer::SubscriberExt, registry::LookupSpan};

    #[derive(Clone, Debug)]
    struct RecordedSpan {
        name: String,
        fields: HashMap<String, String>,
    }

    #[derive(Clone, Default)]
    struct Capture {
        open: Arc<Mutex<HashMap<tracing::Id, RecordedSpan>>>,
        closed: Arc<Mutex<Vec<RecordedSpan>>>,
    }

    impl Capture {
        fn spans(&self) -> Vec<RecordedSpan> {
            self.closed.lock().expect("capture lock poisoned").clone()
        }
    }

    #[derive(Default)]
    struct FieldVisitor {
        fields: HashMap<String, String>,
    }

    impl tracing::field::Visit for FieldVisitor {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
            self.fields
                .insert(field.name().to_owned(), format!("{value:?}"));
        }

        fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
            self.fields
                .insert(field.name().to_owned(), value.to_owned());
        }

        fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
            self.fields
                .insert(field.name().to_owned(), value.to_string());
        }

        fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
            self.fields
                .insert(field.name().to_owned(), value.to_string());
        }

        fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
            self.fields
                .insert(field.name().to_owned(), value.to_string());
        }
    }

    impl<S> Layer<S> for Capture
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_new_span(
            &self,
            attrs: &tracing::span::Attributes<'_>,
            id: &tracing::Id,
            _ctx: Context<'_, S>,
        ) {
            let mut visitor = FieldVisitor::default();
            attrs.record(&mut visitor);
            self.open.lock().expect("capture lock poisoned").insert(
                id.clone(),
                RecordedSpan {
                    name: attrs.metadata().name().to_owned(),
                    fields: visitor.fields,
                },
            );
        }

        fn on_record(
            &self,
            id: &tracing::Id,
            values: &tracing::span::Record<'_>,
            _ctx: Context<'_, S>,
        ) {
            let mut visitor = FieldVisitor::default();
            values.record(&mut visitor);
            if let Some(span) = self.open.lock().expect("capture lock poisoned").get_mut(id) {
                span.fields.extend(visitor.fields);
            }
        }

        fn on_close(&self, id: tracing::Id, _ctx: Context<'_, S>) {
            if let Some(span) = self.open.lock().expect("capture lock poisoned").remove(&id) {
                self.closed
                    .lock()
                    .expect("capture lock poisoned")
                    .push(span);
            }
        }
    }

    #[traced(subsystem = "test", skip())]
    async fn repeated_phases() -> Result<(), &'static str> {
        let _: () = fetch_time!(Ok::<(), &'static str>(()))?;
        let _: () = fetch_time!(Ok::<(), &'static str>(()))?;
        let _: String = parse_time!("plain text".to_owned());
        let _: String = parse_time!("<html></html>".to_owned());
        let _: String = parse_time!("<?xml version=\"1.0\"?>".to_owned());
        let _: Vec<u8> = parse_time!(vec![0_u8, 1, 2, 255]);
        let _: () = parse_time!(Ok::<(), &'static str>(()))?;
        let _: Result<(), &'static str> = parse_time!(Err::<(), &'static str>("parse"));
        let _: () = parse_time!(Ok::<(), &'static str>(()))?;
        Ok(())
    }

    #[traced(subsystem = "test", skip())]
    async fn failed_fetch() -> Result<(), &'static str> {
        let _: () = fetch_time!(Err::<(), &'static str>("fetch"))?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn phase_spans_are_aggregated_and_failures_are_sticky() {
        let capture = Capture::default();
        let subscriber = tracing_subscriber::registry().with(capture.clone());

        let _default = tracing::subscriber::set_default(subscriber);
        assert!(repeated_phases().await.is_ok());
        assert!(failed_fetch().await.is_err());

        let spans = capture.spans();
        let fetch_spans: Vec<_> = spans.iter().filter(|span| span.name == "fetch").collect();
        let parse_spans: Vec<_> = spans.iter().filter(|span| span.name == "parse").collect();
        assert_eq!(fetch_spans.len(), 2);
        assert_eq!(parse_spans.len(), 1);

        let repeated_business = spans
            .iter()
            .find(|span| span.name == "repeated_phases")
            .expect("business span missing");
        assert_eq!(
            repeated_business.fields.get("subsystem"),
            Some(&"test".to_owned())
        );
        assert_eq!(
            repeated_business.fields.get("operation"),
            Some(&"repeated_phases".to_owned())
        );
        assert_eq!(
            repeated_business.fields.get("result"),
            Some(&"success".to_owned())
        );

        let repeated_fetch = fetch_spans
            .iter()
            .find(|span| span.fields.get("phase") == Some(&"fetch".to_owned()))
            .expect("repeated fetch span missing");
        assert_eq!(
            repeated_fetch.fields.get("result"),
            Some(&"success".to_owned())
        );

        let repeated_parse = parse_spans
            .iter()
            .find(|span| span.fields.get("phase") == Some(&"parse".to_owned()))
            .expect("repeated parse span missing");
        assert_eq!(
            repeated_parse.fields.get("result"),
            Some(&"error".to_owned())
        );

        let failed_business = spans
            .iter()
            .find(|span| span.name == "failed_fetch")
            .expect("failed business span missing");
        assert_eq!(
            failed_business.fields.get("result"),
            Some(&"error".to_owned())
        );
    }
}
