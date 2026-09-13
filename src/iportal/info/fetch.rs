use crate::{
    error::MapUnexpectedErr,
    iportal::util::IPortalRequestBuilderExt,
    iportal::{error::IPortalExpired, login::IPortalToken},
    utils::client,
};

pub async fn fetch_info(token: &IPortalToken) -> Result<String, crate::Error<IPortalExpired>> {
    client
        .get("https://iportal.hnu.edu.cn/personal/frontend/data/info")
        .send_with_token(token)
        .await?
        .text()
        .await
        .unexpected_err()
}
