#![doc = include_str!("../docs/test.md")]

use std::sync::LazyLock;

pub static TEST_STU_ID: &str = env!("TEST_STU_ID");

pub static TEST_PASSWORD: &str = env!("TEST_PASSWORD");

pub static TEST_XN: LazyLock<u16> =
    LazyLock::new(|| std::env::var("TEST_XN").unwrap().parse().unwrap());

pub static TEST_XQ: LazyLock<u8> =
    LazyLock::new(|| std::env::var("TEST_XQ").unwrap().parse().unwrap());

pub static TEST_YEAR: LazyLock<u16> =
    LazyLock::new(|| std::env::var("TEST_YEAR").unwrap().parse().unwrap());

pub static TEST_MONTH: LazyLock<u8> =
    LazyLock::new(|| std::env::var("TEST_MONTH").unwrap().parse().unwrap());

pub static TEST_DAY: LazyLock<u8> =
    LazyLock::new(|| std::env::var("TEST_DAY").unwrap().parse().unwrap());
