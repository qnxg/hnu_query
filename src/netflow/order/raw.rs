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
pub struct OrderItem {
    // pub AddTime: String,
    // pub AllowOverTraffic: f64,
    // pub BaseTraffic: f64,
    pub Download: Option<f64>,
    // pub ExtTraffic: f64,
    pub Month: String,
    // pub PayOrderCode: Option<String>,
    /// 1:已支付 0:未支付
    // pub PayState: u32,
    pub RealOverTraffic: f64,
    pub ShouldPay: f64,
    // pub Total: f64,
    pub UpdateTime: String,
    pub Upload: Option<f64>,
    // pub Year: String,
}

pub async fn raw_order_data(
    netflow_token: &NetflowToken,
) -> Result<Vec<OrderItem>, crate::Error<Infallible>> {
    let headers = netflow_token.headers().clone();
    let json_str = client
        .get(NETFLOW_ORDER_URL)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Vec<OrderItem> = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}
