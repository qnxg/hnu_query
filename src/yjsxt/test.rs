use crate::{
    cas::{self},
    test::TestResult,
    yjsxt::login::YjsxtToken,
};

pub async fn get_yjsxt_token() -> TestResult<YjsxtToken> {
    let cas_token = cas::test::get_cas_token().await?;
    let token = YjsxtToken::acquire_by_cas_login(&cas_token).await?;
    Ok(token)
}
