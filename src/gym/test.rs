use crate::{
    cas::login::{AccountIssue, CasToken},
    gym::login::GymToken,
    test::{TEST_PASSWORD, TEST_STU_ID},
};
use std::convert::Infallible;

pub async fn get_gym_token_by_cas_login() -> Result<GymToken, crate::Error<AccountIssue>> {
    let mut cas_token = CasToken::new(TEST_STU_ID, TEST_PASSWORD);
    GymToken::acquire_by_cas_login(&mut cas_token).await
}

pub async fn get_gym_token_by_direct_login() -> Result<GymToken, crate::Error<Infallible>> {
    GymToken::acquire_by_direct_login(TEST_STU_ID, TEST_PASSWORD).await
}

pub async fn get_gym_token() -> GymToken {
    if let Ok(gym_token) = get_gym_token_by_cas_login().await {
        gym_token
    } else {
        get_gym_token_by_direct_login().await.unwrap()
    }
}
