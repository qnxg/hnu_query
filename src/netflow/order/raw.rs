use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use serde_json::Value;
use std::convert::Infallible;

const NETFLOW_ORDER_URL: &str = "http://ll.hnu.edu.cn/api/v1/historyorder/getpagedlist";

pub async fn get_order_list(
    netflow_token: &NetflowToken,
) -> Result<Value, crate::Error<Infallible>> {
    client
        .get(NETFLOW_ORDER_URL)
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
