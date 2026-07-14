mod fetch;
mod parse;

use crate::{
    ai::login::AiToken,
    error::MapUnexpectedErr,
    utils::obs::{fetch_time, parse_time, traced},
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// Token 信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    pub token_name: String,
    pub id: u64,
}

/// 获取 token 列表
///
/// **注意：token 是可以重名的**
///
/// # Arguments
///
/// - `token`: 已登录的 AI 系统的令牌，可以通过 [AiToken::acquire_by_cas_login] 创建
///
/// # Returns
///
/// 返回 token 列表，无 token 时返回空列表
#[traced(subsystem = "ai", skip(token))]
pub async fn get_token_list(token: &AiToken) -> Result<Vec<TokenInfo>, crate::Error<Infallible>> {
    let json_str = fetch_time!(fetch::token_list(token).await)?;
    parse_time!(parse::token_list(&json_str))
}

/// 获取指定 token 的 key
///
/// # Arguments
///
/// - `token`: 已登录的 AI 系统的令牌，可以通过 [AiToken::acquire_by_cas_login] 创建
/// - `id`: token 的 ID
///
/// # Returns
///
/// 返回 key 值
#[traced(subsystem = "ai", skip(token))]
pub async fn get_token_key(token: &AiToken, id: u64) -> Result<String, crate::Error<Infallible>> {
    let json_str = fetch_time!(fetch::token_key(token, id).await)?;
    parse_time!(parse::token_key(&json_str))
}

/// 删除指定 token
///
/// # Arguments
///
/// - `token`: 已登录的 AI 系统的令牌，可以通过 [AiToken::acquire_by_cas_login] 创建
/// - `id`: 要删除的 token 的 ID
#[traced(subsystem = "ai", skip(token))]
pub async fn delete_token(token: &AiToken, id: u64) -> Result<(), crate::Error<Infallible>> {
    let json_str = fetch_time!(fetch::delete_token(token, id).await)?;
    let success = parse_time!(parse::check_action_success(&json_str))?;
    if !success {
        return Err("创建 token 失败，服务器返回 success=false".to_string()).unexpected_err()?;
    }
    Ok(())
}

/// 创建新的 token
///
/// # Arguments
///
/// - `token`: 已登录的 AI 系统的令牌，可以通过 [AiToken::acquire_by_cas_login] 创建
/// - `name`: token 名称
#[traced(subsystem = "ai", skip(token))]
pub async fn create_token(token: &AiToken, name: &str) -> Result<(), crate::Error<Infallible>> {
    let json_str = fetch_time!(fetch::create_token(token, name).await)?;
    let success = parse_time!(parse::check_action_success(&json_str))?;
    if !success {
        return Err("创建 token 失败，服务器返回 success=false".to_string()).unexpected_err()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ai::test::get_ai_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_token_list() -> TestResult<()> {
        let token = get_ai_token().await?;
        let tokens = get_token_list(&token).await?;
        println!("{:#?}", tokens);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_token_key() -> TestResult<()> {
        let token = get_ai_token().await?;
        let tokens = get_token_list(&token).await?;
        if let Some(t) = tokens.first() {
            let key = get_token_key(&token, t.id).await?;
            println!("key: {}", key);
        }
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_token() -> TestResult<()> {
        let token = get_ai_token().await?;
        create_token(&token, "test-token").await?;
        println!("create_token success");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_token() -> TestResult<()> {
        let token = get_ai_token().await?;
        let tokens = get_token_list(&token).await?;
        // 删除 test_create_token 中创建的同名 token
        if let Some(t) = tokens.iter().find(|t| t.token_name == "test-token") {
            delete_token(&token, t.id).await?;
            println!("delete_token success");
        }
        Ok(())
    }
}
