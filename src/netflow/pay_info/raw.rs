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
pub struct PayInfo {
    pub Total: f64,
}

pub async fn raw_pay_info_data(
    netflow_token: &NetflowToken,
) -> Result<PayInfo, crate::Error<Infallible>> {
    let headers = netflow_token.headers().clone();
    let json_str = client
        .get(NETFLOW_PAY_INFO_URL)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}
