use crate::{
    ai::login::AiToken,
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

// maas 平台 token 管理端点
const TOKENS_URL: &str = "https://maas.nscc-cs.cn/api/tokens";
const APPLY_TOKEN_URL: &str = "https://maas.nscc-cs.cn/api/apply-token";

/// Token 信息
#[derive(Debug, Clone, Deserialize)]
pub struct TokenInfo {
    pub token_name: String,
    pub id: u64,
}

pub async fn raw_token_list(token: &AiToken) -> Result<Vec<TokenInfo>, crate::Error<Infallible>> {
    let headers = token.headers().clone();
    let json_str = client
        .get(TOKENS_URL)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;
    let tokens: Vec<TokenInfo> =
        serde_json::from_value(res["data"].clone()).parse_err(&json_str)?;
    Ok(tokens)
}

pub async fn raw_token_key(token: &AiToken, id: u64) -> Result<String, crate::Error<Infallible>> {
    let headers = token.headers().clone();
    let json_str = client
        .get(format!("{}/{}/key", TOKENS_URL, id))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;
    let key = res["data"]["key"].as_str().ok_or(parse_err(&json_str))?;
    Ok(key.to_string())
}

pub async fn raw_delete_token(token: &AiToken, id: u64) -> Result<bool, crate::Error<Infallible>> {
    let headers = token.headers().clone();
    let json_str = client
        .delete(format!("{}/{}", TOKENS_URL, id))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;
    let success = res["success"].as_bool().ok_or(parse_err(&json_str))?;
    Ok(success)
}

pub async fn raw_create_token(
    token: &AiToken,
    name: &str,
) -> Result<bool, crate::Error<Infallible>> {
    let headers = token.headers().clone();
    let json_str = client
        .post(APPLY_TOKEN_URL)
        .headers(headers)
        .json(&serde_json::json!({"name": name, "quota": 2000000}))
        // quota参数似乎没什么用，加上只是为了与网页行为保持一致
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;
    let success = res["success"].as_bool().ok_or(parse_err(&json_str))?;
    Ok(success)
}
