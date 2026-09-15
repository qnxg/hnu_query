use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::client,
};
use chrono::NaiveDate;

pub async fn balance(token: &IPortalToken) -> Result<String, crate::Error<IPortalTokenExpired>> {
    let url = "https://iportal.hnu.edu.cn/hnu/frontend/user/card-balance";
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

pub async fn transaction_records(
    token: &IPortalToken,
    account: String,
    start: NaiveDate,
    end: NaiveDate,
    page_size: u32,
    page: u32,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    let url = format!(
        "https://iportal.hnu.edu.cn/hnu/frontend/user/card-details?query_start={}&query_end={}&page_size={page_size}&page={page}&account={account}",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d"),
    );
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
