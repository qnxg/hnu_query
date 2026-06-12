use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    netflow::login::NetflowToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

const NETFLOW_PAY_INFO_URL: &str = "http://ll.hnu.edu.cn/api/v1/pay/getpayinfo";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct RawPayInfo {
    pub Total: f64,
}

pub async fn get_pay_info(
    netflow_token: &NetflowToken,
) -> Result<RawPayInfo, crate::Error<Infallible>> {
    let json_str = client
        .get(NETFLOW_PAY_INFO_URL)
        .headers(netflow_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;

    let data = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or_else(|| parse_err(&json_str))?;
    Ok(data)
}
