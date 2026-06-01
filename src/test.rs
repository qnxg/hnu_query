#![doc = include_str!("../docs/test.md")]

use std::sync::LazyLock;

pub static TEST_STU_ID: &str = env!("TEST_STU_ID");

pub static TEST_PASSWORD: &str = env!("TEST_PASSWORD");

/// 主要用于发送请求类的测试，该函数会尝试将一个 [Result] 中的值 unwrap，
/// 如果失败则 panic，并打印相关信息
pub fn test_ok<T, E: std::fmt::Debug>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|e| panic!("{}: {:?}", context, e))
}

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

pub static TEST_XN: LazyLock<u16> = LazyLock::new(|| test_env_parse("TEST_XN"));

pub static TEST_XQ: LazyLock<u8> = LazyLock::new(|| test_env_parse("TEST_XQ"));

pub static TEST_YEAR: LazyLock<u16> = LazyLock::new(|| test_env_parse("TEST_YEAR"));

pub static TEST_MONTH: LazyLock<u8> = LazyLock::new(|| test_env_parse("TEST_MONTH"));

pub static TEST_DAY: LazyLock<u8> = LazyLock::new(|| test_env_parse("TEST_DAY"));

pub static TEST_CAS_CACHE: LazyLock<bool> = LazyLock::new(|| {
    let v = env_var("TEST_CAS_CACHE");
    v == "true" || v == "1"
});
