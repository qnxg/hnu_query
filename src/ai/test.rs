use crate::{ai::login::AiToken, cas::test::get_cas_token, test::TestResult};

pub async fn get_ai_token() -> TestResult<AiToken> {
    let cas_token = get_cas_token().await?;
    Ok(AiToken::acquire_by_cas_login(&cas_token).await?)
}
