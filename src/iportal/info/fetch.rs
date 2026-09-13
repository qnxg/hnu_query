use crate::{
    cas, error::MapUnexpectedErr, iportal::login::IPortalToken,
    iportal::util::IPortalRequestBuilderExt, utils::client,
};

const INFO_ENDPOINT: &str = "https://iportal.hnu.edu.cn/personal/frontend/data/info";

pub async fn fetch_info(
    token: &IPortalToken,
) -> Result<String, crate::Error<cas::error::TokenExpired>> {
    let response = client.get(INFO_ENDPOINT).send_with_token(token).await?;

    response.text().await.unexpected_err()
}
