use crate::{
    error::MapUnexpectedErr,
    iportal::{error::IPortalExpired, login::IPortalToken, util::IPortalRequestBuilderExt},
    utils::client,
};

pub async fn fetch_term_info(
    token: &IPortalToken,
    timestamp: i64,
) -> Result<String, crate::Error<IPortalExpired>> {
    client
        .get(format!(
            "https://iportal.hnu.edu.cn/hnu/frontend/term?current_time={timestamp}"
        ))
        .send_with_token(token)
        .await?
        .text()
        .await
        .unexpected_err()
}
