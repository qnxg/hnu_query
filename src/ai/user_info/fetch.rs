use crate::{
    ai::login::AiToken,
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
};
use std::convert::Infallible;

// maas 平台用户信息端点
const USER_INFO_URL: &str = "https://maas.nscc-cs.cn/api/user-info";

pub async fn user_info_data(token: &AiToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(USER_INFO_URL)
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}
