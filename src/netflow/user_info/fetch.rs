use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    netflow::login::NetflowToken,
    utils::client,
};
use std::convert::Infallible;

const NETFLOW_USER_INFO_URL: &str = "http://ll.hnu.edu.cn/api/v1/account/getuserinfo";

pub async fn user_info(netflow_token: &NetflowToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(NETFLOW_USER_INFO_URL)
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
