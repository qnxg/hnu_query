//! 可观测性相关的辅助模块，该模块包装了 [tracing] 的一些操作，
//! 当 `tracing` feature 开启时，本模块的宏和函数转发到 [tracing]；
//! 关闭时全部为 no-op，编译期即被完全消除。
//!
//! 这样其他地方的代码只需要统一调用本模块，无需反复判断是否开启 tracing feature。
#![allow(unused_macros, unused_imports)]

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

// 将 future 挂到未 enter 的 DEBUG span 上，适合 `try_join!` 等并发场景。
// 调用方：`obs::instrument!("name", some_future)`。feature 关闭时原样返回 future。
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
/// 字段必须在创建该 span 时用 `Empty`（或已有初值）声明过，否则该操作会被忽略。
/// feature 关闭时为 no-op。
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

pub(crate) use {
    debug, debug_span, error, info, info_span, instrument, record, trace, warn_evt as warning,
};

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
