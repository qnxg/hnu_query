use crate::{
    cas::{self},
    test::TestResult,
    xgxt::login::XgxtToken,
};

pub async fn get_xgxt_token() -> TestResult<XgxtToken> {
    let cas_token = cas::test::get_cas_token().await?;
    Ok(XgxtToken::acquire_by_cas_login(&cas_token).await?)
}
