use crate::{
    ai::login::AiToken,
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::client,
};
use serde_json::Value;
use std::convert::Infallible;

// maas 平台用户信息端点
const USER_INFO_URL: &str = "https://maas.nscc-cs.cn/api/user-info";

pub async fn raw_user_info_data(token: &AiToken) -> Result<usize, crate::Error<Infallible>> {
    let headers = token.headers().clone();
    let json_str = client
        .get(USER_INFO_URL)
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
    let total_granted = res["data"]["total_granted"]
        .as_u64()
        .ok_or(parse_err(&json_str))? as usize;
    Ok(total_granted)
}
