use crate::{
    ai::login::AiToken,
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    utils::client,
};
use std::convert::Infallible;

// maas 平台 token 管理端点
const TOKENS_URL: &str = "https://maas.nscc-cs.cn/api/tokens";
const APPLY_TOKEN_URL: &str = "https://maas.nscc-cs.cn/api/apply-token";

pub async fn token_list(token: &AiToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(TOKENS_URL)
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn token_key(token: &AiToken, id: u64) -> Result<String, crate::Error<Infallible>> {
    client
        .get(format!("{}/{}/key", TOKENS_URL, id))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn delete_token(token: &AiToken, id: u64) -> Result<String, crate::Error<Infallible>> {
    client
        .delete(format!("{}/{}", TOKENS_URL, id))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn create_token(token: &AiToken, name: &str) -> Result<String, crate::Error<Infallible>> {
    client
        .post(APPLY_TOKEN_URL)
        .headers(token.headers().clone())
        // quota参数似乎没什么用，加上只是为了与网页行为保持一致
        .json(&serde_json::json!({"name": name, "quota": 2000000}))
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
