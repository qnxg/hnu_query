use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    iportal::{
        error::{CheckIPortalTokenExpired, IPortalTokenExpired},
        login::IPortalToken,
    },
    utils::client,
};

const PERSONAL_DATA_ITEMS_URL: &str =
    "https://iportal.hnu.edu.cn/personal/frontend/data/items?type=personal_data";
const PERSONAL_DATA_DETAIL_URL: &str = "https://iportal.hnu.edu.cn/personal/frontend/data/detail";

pub async fn personal_data_query_ids(
    token: &IPortalToken,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    client
        .get(PERSONAL_DATA_ITEMS_URL)
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

pub async fn personal_data_single(
    token: &IPortalToken,
    id: &str,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    let url = format!("{PERSONAL_DATA_DETAIL_URL}?id={id}");
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
