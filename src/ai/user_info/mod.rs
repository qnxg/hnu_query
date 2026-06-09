mod raw;

use crate::ai::{login::AiToken, user_info::raw::raw_user_info_data};
use std::convert::Infallible;

/// 获取用户剩余的 token 额度
///
/// # Parameters
///
/// - `token`: 已登录的 AI 系统的令牌，可以通过 [AiToken::acquire_by_cas_login] 创建
pub async fn get_user_total_granted(token: &AiToken) -> Result<usize, crate::Error<Infallible>> {
    raw_user_info_data(token).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ai::test::get_ai_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_user_total_granted() -> TestResult<()> {
        let token = get_ai_token().await?;
        let total_granted = get_user_total_granted(&token).await?;
        println!("total_granted: {}", total_granted);
        Ok(())
    }
}
