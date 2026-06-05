use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use serde_json::Value;
use std::convert::Infallible;

const NETFLOW_MONTH_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbymonth?month=";
const NETFLOW_DAY_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbyday?day=";

pub async fn get_float_detail_by_month(
    netflow_token: &NetflowToken,
    year: u16,
    month: u8,
) -> Result<Value, crate::Error<Infallible>> {
    let url = format!("{NETFLOW_MONTH_URL}{}-{:0>2}", year, month);
    client
        .get(url)
        .headers(netflow_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .json()
        .await
        .unexpected_err()
}

pub async fn get_float_detail_by_day(
    netflow_token: &NetflowToken,
    year: u16,
    month: u8,
    day: u8,
) -> Result<Value, crate::Error<Infallible>> {
    let url = format!("{NETFLOW_DAY_URL}{}{:0>2}{:0>2}", year, month, day);
    client
        .get(url)
        .headers(netflow_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .json()
        .await
        .unexpected_err()
}
