use crate::{
    cas::login::{AccountIssue, CasToken},
    test::{TEST_PASSWORD, TEST_STU_ID},
    xgxt::login::XgxtToken,
};

pub async fn get_xgxt_token() -> Result<XgxtToken, crate::Error<AccountIssue>> {
    let mut cas_token = CasToken::new(TEST_STU_ID, TEST_PASSWORD);
    let xgxt_token = XgxtToken::acquire_by_cas_login(&mut cas_token).await?;
    Ok(xgxt_token)
}
