use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use serde_json::Value;
use std::convert::Infallible;

const NETFLOW_PAY_INFO_URL: &str = "http://ll.hnu.edu.cn/api/v1/pay/getpayinfo";

pub async fn get_pay_info(netflow_token: &NetflowToken) -> Result<Value, crate::Error<Infallible>> {
    client
        .get(NETFLOW_PAY_INFO_URL)
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
