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
pub struct Detail {
    pub AllDownload: f64,
    pub AllTotal: f64,
    pub AllUpload: f64,
    pub FloatDetailList: Vec<DetailItem>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct DetailItem {
    pub App: String,
    pub Download: f64,
    pub Per: f64,
    pub Total: f64,
    pub Upload: f64,
}

pub async fn raw_month_detail_data(
    netflow_token: &NetflowToken,
    year: u16,
    month: u8,
) -> Result<Detail, crate::Error<Infallible>> {
    let headers = netflow_token.headers().clone();
    let url = format!("{NETFLOW_MONTH_URL}{}-{:0>2}", year, month);
    let json_str = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Detail = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}

pub async fn raw_day_detail_data(
    netflow_token: &NetflowToken,
    year: u16,
    month: u8,
    day: u8,
) -> Result<Detail, crate::Error<Infallible>> {
    let headers = netflow_token.headers().clone();
    let url = format!("{NETFLOW_DAY_URL}{}{:0>2}{:0>2}", year, month, day);
    let json_str = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Detail = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}
