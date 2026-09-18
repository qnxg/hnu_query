use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    iportal::{
        error::{CheckIPortalTokenExpired, IPortalTokenExpired},
        login::IPortalToken,
    },
    utils::client,
};

const INFO_URL: &str = "https://iportal.hnu.edu.cn/personal/frontend/data/info";

pub async fn info(token: &IPortalToken) -> Result<String, crate::Error<IPortalTokenExpired>> {
    client
        .get(INFO_URL)
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .iportal_token_expired()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
