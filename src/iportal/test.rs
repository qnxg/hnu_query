#[cfg(test)]
use crate::{cas, iportal::login::IPortalToken, test::TestResult};

pub async fn get_iportal_token() -> TestResult<IPortalToken> {
    let cas_token = cas::test::get_cas_token().await?;
    Ok(IPortalToken::acquire_by_cas_login(&cas_token).await?)
}
