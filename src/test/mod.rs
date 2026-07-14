#![doc = include_str!("../../docs/test.md")]
mod obs;

use std::sync::LazyLock;

pub use obs::init_tracing;

/// 测试函数的返回类型，可直接使用 `?` 传播错误
pub type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|e| panic!("missing environment variable {key}: {e}"))
}

pub fn test_env_parse<T: std::str::FromStr>(key: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    env_var(key)
        .parse()
        .unwrap_or_else(|e| panic!("invalid value for {key}: {e:?}"))
}

pub static TEST_STU_ID: &str = env!("TEST_STU_ID");
pub static TEST_PASSWORD: &str = env!("TEST_PASSWORD");
pub static TEST_XN: LazyLock<u16> = LazyLock::new(|| test_env_parse("TEST_XN"));
pub static TEST_XQ: LazyLock<u8> = LazyLock::new(|| test_env_parse("TEST_XQ"));
pub static TEST_YEAR: LazyLock<u16> = LazyLock::new(|| test_env_parse("TEST_YEAR"));
pub static TEST_MONTH: LazyLock<u8> = LazyLock::new(|| test_env_parse("TEST_MONTH"));
pub static TEST_DAY: LazyLock<u8> = LazyLock::new(|| test_env_parse("TEST_DAY"));
pub static TEST_CAS_CACHE: LazyLock<bool> = LazyLock::new(|| {
    let v = env_var("TEST_CAS_CACHE");
    v == "true" || v == "1"
});
