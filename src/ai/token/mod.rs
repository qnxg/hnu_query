mod raw;

use crate::{
    ai::{login::AiToken, token::raw::*},
    error::parse_err,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// Token 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token_name: String,
    pub id: u64,
}

/// 获取 token 列表
///
/// **注意：token 是可以重名的**
///
/// # Parameters
///
/// - `token`: 已登录的 [AiToken]
///
/// # Returns
///
/// 返回 token 列表，无 token 时返回空列表
pub async fn get_token_list(token: &AiToken) -> Result<Vec<TokenInfo>, crate::Error<Infallible>> {
    let raw_data = raw_token_list(token).await?;
    let arr = raw_data["data"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    arr.iter()
        .map(|item| {
            Ok(TokenInfo {
                token_name: item["token_name"]
                    .as_str()
                    .ok_or(parse_err(&serde_json::to_string(item).unwrap_or_default()))?
                    .to_string(),
                id: item["id"]
                    .as_u64()
                    .ok_or(parse_err(&serde_json::to_string(item).unwrap_or_default()))?,
            })
        })
        .collect()
}

/// 获取指定 token 的 key
///
/// # Parameters
///
/// - `token`: 已登录的 [AiToken]
/// - `id`: token 的 ID
///
/// # Returns
///
/// 返回 key 值
pub async fn get_token_key(token: &AiToken, id: u64) -> Result<String, crate::Error<Infallible>> {
    let raw_data = raw_token_key(token, id).await?;
    let key = raw_data["data"]["key"].as_str().ok_or(parse_err(
        &serde_json::to_string(&raw_data).unwrap_or_default(),
    ))?;
    Ok(key.to_string())
}

/// 删除指定 token
///
/// **!!! 删除成功后应重新获取 token 列表**
///
/// # Parameters
///
/// - `token`: 已登录的 [AiToken]
/// - `id`: 要删除的 token 的 ID
pub async fn delete_token(token: &AiToken, id: u64) -> Result<(), crate::Error<Infallible>> {
    let _raw_data = raw_delete_token(token, id).await?;
    Ok(())
}

/// 创建新的 token
///
/// **!!! 创建成功后应重新获取 token 列表**
///
/// # Parameters
///
/// - `token`: 已登录的 [AiToken]
/// - `name`: token 名称
pub async fn create_token(token: &AiToken, name: &str) -> Result<(), crate::Error<Infallible>> {
    let _raw_data = raw_create_token(token, name).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::test::get_ai_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_token_list() {
        let token = get_ai_token().await.unwrap();
        let tokens = get_token_list(&token).await.unwrap();
        println!("{:#?}", tokens);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_token_key() {
        let token = get_ai_token().await.unwrap();
        let tokens = get_token_list(&token).await.unwrap();
        if let Some(t) = tokens.first() {
            let key = get_token_key(&token, t.id).await.unwrap();
            println!("key: {}", key);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_token() {
        let token = get_ai_token().await.unwrap();
        create_token(&token, "test-token").await.unwrap();
        println!("create_token success");
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_token() {
        let token = get_ai_token().await.unwrap();
        let tokens = get_token_list(&token).await.unwrap();
        // 删除 test_create_token 中创建的同名 token
        if let Some(t) = tokens.iter().find(|t| t.token_name == "test-token") {
            delete_token(&token, t.id).await.unwrap();
            println!("delete_token success");
        }
    }
}
