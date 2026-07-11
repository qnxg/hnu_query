use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use std::convert::Infallible;

const NETFLOW_MONTH_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbymonth?month=";
const NETFLOW_DAY_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbyday?day=";

async fn detail(
    netflow_token: &NetflowToken,
    url: String,
) -> Result<String, crate::Error<Infallible>> {
    client
        .get(url)
        .headers(netflow_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}

pub async fn detail_by_month(
    netflow_token: &NetflowToken,
    year: u16,
    month: u8,
) -> Result<String, crate::Error<Infallible>> {
    let url = format!("{NETFLOW_MONTH_URL}{}-{:0>2}", year, month);
    detail(netflow_token, url).await
}

pub async fn detail_by_day(
    netflow_token: &NetflowToken,
    year: u16,
    month: u8,
    day: u8,
) -> Result<String, crate::Error<Infallible>> {
    let url = format!("{NETFLOW_DAY_URL}{}{:0>2}{:0>2}", year, month, day);
    detail(netflow_token, url).await
}
