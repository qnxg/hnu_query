use crate::{
    ca::login::CaToken,
    cas::{self},
    test::TestResult,
};

pub async fn get_ca_token() -> TestResult<CaToken> {
    let cas_token = cas::test::get_cas_token().await?;
    Ok(CaToken::acquire_by_cas_login(&cas_token).await?)
}
