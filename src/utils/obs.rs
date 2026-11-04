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
            let __obs_result =
                $crate::utils::obs::measure_phase($crate::utils::obs::Phase::Fetch, async {
                    $expr
                })
                .await;
            $crate::utils::obs::record_phase_result($crate::utils::obs::Phase::Fetch, {
                use $crate::utils::obs::PhaseOutcome as _;
                (&__obs_result).phase_failed()
            });
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
            let __obs_result =
                $crate::utils::obs::measure_phase($crate::utils::obs::Phase::Parse, async {
                    $expr
                })
                .await;
            $crate::utils::obs::record_phase_result($crate::utils::obs::Phase::Parse, {
                use $crate::utils::obs::PhaseOutcome as _;
                (&__obs_result).phase_failed()
            });
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
#[derive(Clone, Copy, Debug)]
pub enum Phase {
    Fetch,
    Parse,
}

#[cfg(feature = "tracing")]
#[derive(Default)]
struct PhaseState {
    fetch_span: Option<tracing::Span>,
    parse_span: Option<tracing::Span>,
    fetch_ms: u64,
    parse_ms: u64,
    fetch_failed: bool,
    parse_failed: bool,
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

/// 由 [`traced`] 注入：函数结束（含 `?` 提前返回）时把累计耗时和阶段状态写入 span。
pub struct FlushPhaseTimersOnDrop;

impl Drop for FlushPhaseTimersOnDrop {
    fn drop(&mut self) {
        #[cfg(feature = "tracing")]
        flush_phase_timers_to_current_span();
    }
}

#[cfg(feature = "tracing")]
fn flush_phase_timers_to_current_span() {
    let Ok((fetch_ms, parse_ms, fetch_span, parse_span, fetch_failed, parse_failed)) = PHASE_TIMERS
        .try_with(|c| {
            let c = c.borrow();
            (
                c.fetch_ms,
                c.parse_ms,
                c.fetch_span.clone(),
                c.parse_span.clone(),
                c.fetch_failed,
                c.parse_failed,
            )
        })
    else {
        return;
    };

    let span = tracing::Span::current();
    span.record("fetch_ms", fetch_ms);
    span.record("parse_ms", parse_ms);

    if let Some(span) = fetch_span {
        finish_phase_span(&span, fetch_ms, fetch_failed);
    }
    if let Some(span) = parse_span {
        finish_phase_span(&span, parse_ms, parse_failed);
    }
}

#[cfg(feature = "tracing")]
fn finish_phase_span(span: &tracing::Span, duration_ms: u64, failed: bool) {
    span.record("duration_ms", duration_ms);
    span.record("err", failed);
    span.record("result", if failed { "error" } else { "success" });
    if failed {
        span.in_scope(|| tracing::error!("phase_error"));
    }
}

#[cfg(feature = "tracing")]
/// 获取当前调用内复用的阶段 Span。
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
                            duration_ms = tracing::field::Empty,
                            err = tracing::field::Empty,
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
                            duration_ms = tracing::field::Empty,
                            err = tracing::field::Empty,
                            result = tracing::field::Empty,
                        )
                    })
                    .clone(),
            }
        })
        .unwrap_or_else(|_| tracing::Span::none())
}

#[cfg(feature = "tracing")]
/// 阶段计时 guard；异常或取消时由 `Drop` 补记耗时。
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

    pub fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        add_phase_ms(self.phase, self.started_at.elapsed().as_millis() as u64);
    }
}

#[cfg(feature = "tracing")]
impl Drop for PhaseTimerGuard {
    fn drop(&mut self) {
        self.finish();
    }
}

#[cfg(feature = "tracing")]
/// 执行阶段逻辑并累计耗时；调用方通过 [`record_phase_result`] 回填失败状态。
pub async fn measure_phase<F>(phase: Phase, future: F) -> F::Output
where
    F: Future,
{
    use tracing::Instrument;

    let span = phase_span(phase);
    let mut guard = PhaseTimerGuard::new(phase);
    let result = future.instrument(span).await;
    guard.finish();
    result
}

/// 判断阶段表达式的输出是否代表失败。
///
/// [`Result`] 使用错误状态；其他值视为成功。这里通过接收者自动引用区分两类输出，
/// 使 [`fetch_time!`] / [`parse_time!`] 不要求输出实现 `Debug`。
#[cfg(feature = "tracing")]
pub trait PhaseOutcome {
    fn phase_failed(self) -> bool;
}

#[cfg(feature = "tracing")]
impl<T, E> PhaseOutcome for &Result<T, E> {
    fn phase_failed(self) -> bool {
        self.is_err()
    }
}

#[cfg(feature = "tracing")]
impl<T> PhaseOutcome for &&T {
    fn phase_failed(self) -> bool {
        false
    }
}

#[cfg(feature = "tracing")]
pub fn record_phase_result(phase: Phase, failed: bool) {
    let _ = PHASE_TIMERS.try_with(|c| {
        let mut c = c.borrow_mut();
        match phase {
            Phase::Fetch => c.fetch_failed |= failed,
            Phase::Parse => c.parse_failed |= failed,
        }
    });
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
