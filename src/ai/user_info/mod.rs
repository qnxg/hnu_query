mod fetch;
mod parse;

use crate::ai::login::AiToken;
use std::convert::Infallible;

/// 获取用户剩余的 token 额度
///
/// # Parameters
///
/// - `token`: 已登录的 AI 系统的令牌，可以通过 [AiToken::acquire_by_cas_login] 创建
pub async fn get_user_remaining_quota(token: &AiToken) -> Result<usize, crate::Error<Infallible>> {
    let json_str = fetch::user_info_data(token).await?;
    parse::remaining_quota(&json_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ai::test::get_ai_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_user_remaining_quota() -> TestResult<()> {
        let token = get_ai_token().await?;
        let remaining = get_user_remaining_quota(&token).await?;
        println!("remaining_quota: {}", remaining);
        Ok(())
    }
}
