use crate::{
    cas::error::TokenExpired,
    error::MapUnexpectedErr,
    iportal::{login::IPortalToken, util::IPortalRequestBuilderExt},
    utils::client,
};

pub async fn fetch_term_info(
    token: &IPortalToken,
    timestamp: i64,
) -> Result<String, crate::Error<TokenExpired>> {
    let url = format!("https://iportal.hnu.edu.cn/hnu/frontend/term?current_time={timestamp}");
    let response = client.get(url).send_with_token(token).await?;

    response.text().await.unexpected_err()
}
