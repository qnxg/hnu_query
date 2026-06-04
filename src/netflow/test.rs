use crate::{
    cas::{self},
    netflow::login::NetflowToken,
    test::TestResult,
};

pub async fn get_netflow_token() -> TestResult<NetflowToken> {
    let cas_token = cas::test::get_cas_token().await?;
    Ok(NetflowToken::acquire_by_cas_login(&cas_token).await?)
}
