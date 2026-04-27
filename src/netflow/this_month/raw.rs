use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    netflow::login::NetflowToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

const THIS_MONTH_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/gettrafficinfobythismonth";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct ThisMonthInfo {
    pub allBasePackageAmount: f64,
    pub allExtendPackageAmount: f64,
    pub allTraffic: String,
    pub basePackageUsed: f64,
    pub basePackageUsedPer: f64,
    pub downloadTraffic: String,
    pub extendPackageUsed: f64,
    pub extendPackageUsedPer: f64,
    pub surplusBasePackage: f64,
    pub surplusExtendPackage: f64,
    pub uploadTraffic: String,
}

/// 本月流量数据
pub async fn raw_this_month_data(
    netflow_token: &NetflowToken,
) -> Result<ThisMonthInfo, crate::Error<Infallible>> {
    let headers = netflow_token.headers().clone();
    let json_str = client
        .get(THIS_MONTH_URL)
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
