use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    netflow::login::NetflowToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

const NETFLOW_MONTH_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbymonth?month=";
const NETFLOW_DAY_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbyday?day=";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct RawDetail {
    pub AllDownload: f64,
    pub AllTotal: f64,
    pub AllUpload: f64,
    pub FloatDetailList: Vec<RawDetailItem>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct RawDetailItem {
    pub App: String,
    pub Download: f64,
    pub Per: f64,
    pub Total: f64,
    pub Upload: f64,
}

async fn raw_detail(
    netflow_token: &NetflowToken,
    url: String,
) -> Result<RawDetail, crate::Error<Infallible>> {
    let json_str = client
        .get(url)
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

pub async fn get_float_detail_by_month(
    netflow_token: &NetflowToken,
    year: u16,
    month: u8,
) -> Result<RawDetail, crate::Error<Infallible>> {
    let url = format!("{NETFLOW_MONTH_URL}{}-{:0>2}", year, month);
    raw_detail(netflow_token, url).await
}

pub async fn get_float_detail_by_day(
    netflow_token: &NetflowToken,
    year: u16,
    month: u8,
    day: u8,
) -> Result<RawDetail, crate::Error<Infallible>> {
    let url = format!("{NETFLOW_DAY_URL}{}{:0>2}{:0>2}", year, month, day);
    raw_detail(netflow_token, url).await
}
