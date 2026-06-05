use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use serde_json::Value;
use std::convert::Infallible;

const THIS_MONTH_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/gettrafficinfobythismonth";

pub async fn get_traffic_info_by_this_month(
    netflow_token: &NetflowToken,
) -> Result<Value, crate::Error<Infallible>> {
    client
        .get(THIS_MONTH_URL)
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
