use crate::{
    ca::login::CaToken,
    cas::login::{AccountIssue, CasToken},
    test::{TEST_PASSWORD, TEST_STU_ID},
};

pub async fn get_ca_token() -> Result<CaToken, crate::Error<AccountIssue>> {
    let mut cas_token = CasToken::new(TEST_STU_ID, TEST_PASSWORD);
    CaToken::acquire_by_cas_login(&mut cas_token).await
}
