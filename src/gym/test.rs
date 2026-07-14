use crate::{
    cas::{self},
    gym::login::GymToken,
    test::{TEST_PASSWORD, TEST_STU_ID, TestResult, init_tracing},
};

use std::convert::Infallible;

pub async fn get_gym_token_by_cas_login() -> TestResult<GymToken> {
    let cas_token = cas::test::get_cas_token().await?;
    Ok(GymToken::acquire_by_cas_login(&cas_token).await?)
}

pub async fn get_gym_token_by_direct_login() -> Result<GymToken, crate::Error<Infallible>> {
    init_tracing();
    GymToken::acquire_by_direct_login(TEST_STU_ID, TEST_PASSWORD).await
}

pub async fn get_gym_token() -> TestResult<GymToken> {
    if let Ok(gym_token) = get_gym_token_by_direct_login().await {
        Ok(gym_token)
    } else {
        get_gym_token_by_cas_login().await
    }
}
