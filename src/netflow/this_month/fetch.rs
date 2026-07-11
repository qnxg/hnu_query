use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use std::convert::Infallible;

const THIS_MONTH_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/gettrafficinfobythismonth";

pub async fn this_month_info(
    netflow_token: &NetflowToken,
) -> Result<String, crate::Error<Infallible>> {
    client
        .get(THIS_MONTH_URL)
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
