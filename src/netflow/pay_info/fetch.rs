use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use std::convert::Infallible;

const NETFLOW_PAY_INFO_URL: &str = "http://ll.hnu.edu.cn/api/v1/pay/getpayinfo";

pub async fn pay_info(netflow_token: &NetflowToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(NETFLOW_PAY_INFO_URL)
        .headers(netflow_token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
