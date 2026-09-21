use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    iportal::{
        error::{CheckIPortalTokenExpired, IPortalTokenExpired},
        login::IPortalToken,
    },
    utils::client,
};

const TERM_URL: &str = "https://iportal.hnu.edu.cn/hnu/frontend/term";

pub async fn term_info(
    token: &IPortalToken,
    timestamp: i64,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    let url = format!("{TERM_URL}?current_time={timestamp}");
    client
        .get(url)
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
