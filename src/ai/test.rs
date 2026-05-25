use crate::{
    ai::login::AiToken,
    cas::{login::AccountIssue, test::get_cas_token},
};

pub async fn get_ai_token() -> Result<AiToken, crate::Error<AccountIssue>> {
    let cas_token = get_cas_token().await?;
    AiToken::acquire_by_cas_login(&cas_token).await
}
