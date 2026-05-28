use crate::{
    ai::login::AiToken,
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::client,
};
use serde_json::Value;
use std::convert::Infallible;

// maas 平台 token 管理端点
const TOKENS_URL: &str = "https://maas.nscc-cs.cn/api/tokens";
const APPLY_TOKEN_URL: &str = "https://maas.nscc-cs.cn/api/apply-token";

pub async fn raw_token_list(token: &AiToken) -> Result<Value, crate::Error<Infallible>> {
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
    Ok(res)
}

pub async fn raw_token_key(token: &AiToken, id: u64) -> Result<Value, crate::Error<Infallible>> {
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
    Ok(res)
}

pub async fn raw_delete_token(token: &AiToken, id: u64) -> Result<Value, crate::Error<Infallible>> {
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
    Ok(res)
}

pub async fn raw_create_token(
    token: &AiToken,
    name: &str,
) -> Result<Value, crate::Error<Infallible>> {
    let headers = token.headers().clone();
    let json_str = client
        .post(APPLY_TOKEN_URL)
        .headers(headers)
        .json(&serde_json::json!({"name": name, "quota": 2000000}))
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;
    Ok(res)
}
