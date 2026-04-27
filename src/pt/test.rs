use crate::{
    cas::login::{AccountIssue, CasToken},
    pt::login::PtToken,
    test::{TEST_PASSWORD, TEST_STU_ID},
};

pub async fn get_pt_token() -> Result<PtToken, crate::Error<AccountIssue>> {
    let mut cas_token = CasToken::new(TEST_STU_ID, TEST_PASSWORD);
    let token = PtToken::acquire_by_cas_login(&mut cas_token).await?;
    Ok(token)
}
