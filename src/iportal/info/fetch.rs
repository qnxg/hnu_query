use crate::{
    error::MapUnexpectedErr,
    iportal::util::IPortalRequestBuilderExt,
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::client,
};

pub async fn fetch_info(token: &IPortalToken) -> Result<String, crate::Error<IPortalTokenExpired>> {
    client
        .get("https://iportal.hnu.edu.cn/personal/frontend/data/info")
        .send_with_token(token)
        .await?
        .text()
        .await
        .unexpected_err()
}
