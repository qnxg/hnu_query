use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    iportal::{
        error::{CheckIPortalTokenExpired, IPortalTokenExpired},
        login::IPortalToken,
    },
    utils::client,
};
use chrono::NaiveDate;

const CARD_BALANCE_URL: &str = "https://iportal.hnu.edu.cn/hnu/frontend/user/card-balance";
const CARD_DETAILS_URL: &str = "https://iportal.hnu.edu.cn/hnu/frontend/user/card-details";

pub async fn balance(token: &IPortalToken) -> Result<String, crate::Error<IPortalTokenExpired>> {
    client
        .get(CARD_BALANCE_URL)
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
        "{CARD_DETAILS_URL}?query_start={}&query_end={}&page_size={page_size}&page={page}&account={account}",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d"),
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
