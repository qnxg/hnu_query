use crate::{
    cas::{self},
    hdjw::login::HdjwToken,
};
use std::sync::LazyLock;

pub static TEST_HDJW_JX0404ID: &str = env!("TEST_HDJW_JX0404ID");

pub static TEST_HDJW_BUILDING_ID: &str = env!("TEST_HDJW_BUILDING_ID");

pub static TEST_HDJW_WEEK: LazyLock<u8> =
    LazyLock::new(|| std::env::var("TEST_HDJW_WEEK").unwrap().parse().unwrap());

pub static TEST_HDJW_DAY_OF_WEEK: LazyLock<u8> = LazyLock::new(|| {
    std::env::var("TEST_HDJW_DAY_OF_WEEK")
        .unwrap()
        .parse()
        .unwrap()
});

pub static TEST_HDJW_TIME: LazyLock<Vec<u8>> = LazyLock::new(|| {
    std::env::var("TEST_HDJW_TIME")
        .unwrap()
        .split(',')
        .map(|x| x.parse().unwrap())
        .collect()
});

pub async fn get_hdjw_token() -> HdjwToken {
    let cas_token = cas::test::get_cas_token().await.unwrap();
    HdjwToken::acquire_by_cas_login(&cas_token).await.unwrap()
}
