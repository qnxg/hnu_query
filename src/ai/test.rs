use crate::{
    ai::login::AiToken,
    cas::login::{AccountIssue, CasToken},
    test::{TEST_PASSWORD, TEST_STU_ID},
};

pub async fn get_ai_token() -> Result<AiToken, crate::Error<AccountIssue>> {
    let mut cas_token = CasToken::new_test(TEST_STU_ID, TEST_PASSWORD);
    AiToken::acquire_by_cas_login(&mut cas_token).await
}
