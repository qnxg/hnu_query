use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::client,
};

pub async fn info(token: &IPortalToken) -> Result<String, crate::Error<IPortalTokenExpired>> {
    let url = "https://iportal.hnu.edu.cn/personal/frontend/data/info";
    client
        .get(url)
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
