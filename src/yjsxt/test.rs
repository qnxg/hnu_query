use crate::{
    cas::{self},
    test::{TestResult, test_env_parse},
    yjsxt::login::YjsxtToken,
};
use std::sync::LazyLock;

pub static TEST_YJSXT_SEMESTER_ID: LazyLock<u16> =
    LazyLock::new(|| test_env_parse("TEST_YJSXT_SEMESTER_ID"));

pub async fn get_yjsxt_token() -> TestResult<YjsxtToken> {
    let cas_token = cas::test::get_cas_token().await?;
    let token = YjsxtToken::acquire_by_cas_login(&cas_token).await?;
    Ok(token)
}
