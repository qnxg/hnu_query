use crate::{
    error::MapUnexpectedErr,
    iportal::{error::IPortalExpired, login::IPortalToken, util::IPortalRequestBuilderExt},
    utils::client,
};

pub async fn fetch_balance(token: &IPortalToken) -> Result<String, crate::Error<IPortalExpired>> {
    client
        .get("https://iportal.hnu.edu.cn/hnu/frontend/user/card-balance")
        .send_with_token(token)
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn fetch_card_transaction_records(
    token: &IPortalToken,
    start: String,
    end: String,
    pagesize: Option<u32>,
    page: Option<u32>,
    account: &str,
) -> Result<String, crate::Error<IPortalExpired>> {
    client.get(format!(
        "https://iportal.hnu.edu.cn/hnu/frontend/user/card-details?query_start={}&query_end={}&page_size={}&page={}&account={}",
        start,
        end,
        pagesize.unwrap_or(10),
        page.unwrap_or(1),
        account
    )).send_with_token(token).await?.text().await.unexpected_err()
}
