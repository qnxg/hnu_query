use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    iportal::{
        error::{CheckIPortalTokenExpired, IPortalTokenExpired},
        login::IPortalToken,
    },
    utils::client,
};

pub async fn apply_list(
    token: &IPortalToken,
    page: u32,
    page_size: u32,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    let url = format!(
        "https://iportal.hnu.edu.cn/personal/frontend/task/apply?type=all&page={page}&pageSize={page_size}"
    );
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
