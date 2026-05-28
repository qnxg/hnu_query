mod raw;

use crate::{
    ai::{login::AiToken, user_info::raw::raw_user_info_data},
    error::parse_err,
};
use std::convert::Infallible;

/// 获取用户剩余 token
///
/// # Parameters
///
/// - `token`: 已登录的 [AiToken]
///
/// # Returns
///
/// 返回 `data.total_granted` 的值（剩余 token）
pub async fn get_user_total_granted(token: &AiToken) -> Result<i64, crate::Error<Infallible>> {
    let raw_data = raw_user_info_data(token).await?;
    let total_granted = raw_data["data"]["total_granted"].as_i64().ok_or(parse_err(
        &serde_json::to_string(&raw_data).unwrap_or_default(),
    ))?;
    Ok(total_granted)
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
