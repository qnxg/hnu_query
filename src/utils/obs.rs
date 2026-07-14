//! 可观测性相关的辅助模块，该模块包装了 [tracing] 的一些操作，
//! 当 `tracing` feature 开启时，本模块的宏和函数转发到 [tracing]；
//! 关闭时全部为 no-op，编译期即被完全消除。
//!
//! # 阶段计时（fetch / parse）
//!
//! 公开 API 使用 [`traced`] 后，函数体内用 [`fetch_time!`] / [`parse_time!`]
//! 包裹各阶段表达式；耗时会累加到 task-local 计时器，并在函数结束时写入
//! span 字段 `fetch_ms` / `parse_ms`（毫秒）。
//!
//! ```ignore
//! use crate::utils::obs::{fetch_time, parse_time, traced};
//!
//! #[traced(subsystem = "ai", skip(token))]
//! pub async fn get_token_list(token: &AiToken) -> Result<...> {
//!     let json_str = fetch_time!(fetch::token_list(token).await)?;
//!     parse_time!(parse::token_list(&json_str))
//! }
//! ```
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
            let __obs_start = ::std::time::Instant::now();
            let __obs_result = $expr;
            $crate::utils::obs::add_fetch_ms(__obs_start.elapsed().as_millis() as u64);
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
            let __obs_start = ::std::time::Instant::now();
            let __obs_result = $expr;
            $crate::utils::obs::add_parse_ms(__obs_start.elapsed().as_millis() as u64);
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
#[derive(Default)]
struct PhaseAccum {
    fetch_ms: u64,
    parse_ms: u64,
}

#[cfg(feature = "tracing")]
tokio::task_local! {
    static PHASE_TIMERS: std::cell::RefCell<PhaseAccum>;
}

/// 由 [`traced`] 注入：在函数体作用域内启用阶段计时器。
pub async fn with_phase_timers<F: Future>(f: F) -> F::Output {
    #[cfg(feature = "tracing")]
    {
        PHASE_TIMERS
            .scope(std::cell::RefCell::new(PhaseAccum::default()), f)
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
    let Ok(accum) = PHASE_TIMERS.try_with(|c| {
        let c = c.borrow();
        (c.fetch_ms, c.parse_ms)
    }) else {
        return;
    };
    let span = tracing::Span::current();
    span.record("fetch_ms", accum.0);
    span.record("parse_ms", accum.1);
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

/// feature 关闭时 span 宏返回的占位 guard。
///
/// 存在的唯一作用是让 `let _guard = obs::debug_span!(...)` 在 feature 关闭时
/// 绑定到一个非 `()` 类型，避免 `clippy::let_unit_value` 警告。
#[cfg(not(feature = "tracing"))]
pub struct NoopSpanGuard;

// 实现 `Drop` 是为了让 `drop(_s)` 在 feature 关闭时也能通过 clippy 的
// `drop_non_drop` 检查（与 `EnteredSpan` 的 API 保持一致）。
#[cfg(not(feature = "tracing"))]
impl Drop for NoopSpanGuard {
    fn drop(&mut self) {}
}
