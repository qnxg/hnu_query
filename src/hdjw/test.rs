use crate::{
    cas::{self},
    hdjw::login::HdjwToken,
    test::{TestResult, test_env_parse},
};

use std::sync::LazyLock;

pub static TEST_HDJW_JX0404ID: &str = env!("TEST_HDJW_JX0404ID");

pub static TEST_HDJW_BUILDING_ID: &str = env!("TEST_HDJW_BUILDING_ID");

pub static TEST_HDJW_WEEK: LazyLock<u8> = LazyLock::new(|| test_env_parse("TEST_HDJW_WEEK"));

pub static TEST_HDJW_DAY_OF_WEEK: LazyLock<u8> =
    LazyLock::new(|| test_env_parse("TEST_HDJW_DAY_OF_WEEK"));

pub static TEST_HDJW_TIME: LazyLock<Vec<u8>> = LazyLock::new(|| {
    std::env::var("TEST_HDJW_TIME")
        .unwrap_or_else(|e| panic!("missing environment variable TEST_HDJW_TIME: {e}"))
        .split(',')
        .map(|x| {
            x.parse()
                .unwrap_or_else(|e| panic!("invalid TEST_HDJW_TIME segment {x:?}: {e}"))
        })
        .collect()
});

pub async fn get_hdjw_token() -> TestResult<HdjwToken> {
    let cas_token = cas::test::get_cas_token().await?;
    Ok(HdjwToken::acquire_by_cas_login(&cas_token).await?)
}
