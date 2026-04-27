use crate::{
    cas::login::{AccountIssue, CasToken},
    netflow::login::NetflowToken,
    test::{TEST_PASSWORD, TEST_STU_ID},
};

pub async fn get_netflow_token() -> Result<NetflowToken, crate::Error<AccountIssue>> {
    let mut cas_token = CasToken::new(TEST_STU_ID, TEST_PASSWORD);
    let netflow_token = NetflowToken::acquire_by_cas_login(&mut cas_token).await?;
    Ok(netflow_token)
}
