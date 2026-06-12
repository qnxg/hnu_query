use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    netflow::login::NetflowToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

const NETFLOW_ORDER_URL: &str = "http://ll.hnu.edu.cn/api/v1/historyorder/getpagedlist";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct RawOrderItem {
    pub Download: Option<f64>,
    pub Month: String,
    pub RealOverTraffic: f64,
    pub ShouldPay: f64,
    pub UpdateTime: String,
    pub Upload: Option<f64>,
}

pub async fn get_order_list(
    netflow_token: &NetflowToken,
) -> Result<Vec<RawOrderItem>, crate::Error<Infallible>> {
    let json_str = client
        .get(NETFLOW_ORDER_URL)
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
