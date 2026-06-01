mod raw;

use crate::ai::{login::AiToken, user_info::raw::raw_user_info_data};
use std::convert::Infallible;

/// 获取用户总计授予额度
///
/// # Parameters
///
/// - `token`: 已登录的 AI 系统的令牌，可以通过 [AiToken::acquire_by_cas_login] 创建
///
/// # Returns
///
/// 返回 `data.total_granted` 的值（剩余 token）
pub async fn get_user_total_granted(token: &AiToken) -> Result<i64, crate::Error<Infallible>> {
    raw_user_info_data(token).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::test::get_ai_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_user_total_granted() {
        let token = get_ai_token().await.unwrap();
        let total_granted = get_user_total_granted(&token).await.unwrap();
        println!("total_granted: {}", total_granted);
    }
}