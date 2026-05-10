use crate::{
    cas::login::{AccountIssue, CasToken},
    test::{TEST_PASSWORD, TEST_STU_ID},
    yjsxt::login::YjsxtToken,
};

pub async fn get_yjsxt_token() -> Result<YjsxtToken, crate::Error<AccountIssue>> {
    let mut cas_token = CasToken::new(TEST_STU_ID, TEST_PASSWORD);
    YjsxtToken::acquire_by_cas_login(&mut cas_token).await
}
