use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use std::convert::Infallible;

const NETFLOW_ORDER_URL: &str = "http://ll.hnu.edu.cn/api/v1/historyorder/getpagedlist";

pub async fn order(netflow_token: &NetflowToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(NETFLOW_ORDER_URL)
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
