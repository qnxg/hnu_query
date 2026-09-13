use crate::{
    error::MapUnexpectedErr,
    iportal::{error::IPortalTokenExpired, login::IPortalToken, util::IPortalRequestBuilderExt},
    utils::client,
};

pub async fn fetch_apply_list(
    token: &IPortalToken,
    page: i32,
    page_size: i32,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    client
        .get(format!(
            "https://iportal.hnu.edu.cn/personal/frontend/task/apply?type=all&page={}&pageSize={}",
            page, page_size
        ))
        .send_with_token(token)
        .await?
        .text()
        .await
        .unexpected_err()
}
