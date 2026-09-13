use crate::{
    error::MapUnexpectedErr,
    iportal::{error::IPortalExpired, login::IPortalToken, util::IPortalRequestBuilderExt},
    utils::client,
};

pub async fn fetch_apply_list(
    token: &IPortalToken,
    page: i32,
    page_size: i32,
) -> Result<String, crate::Error<IPortalExpired>> {
    let url = format!(
        "https://iportal.hnu.edu.cn/personal/frontend/task/apply?type=all&page={}&pageSize={}",
        page, page_size
    );
    let response = client.get(url).send_with_token(token).await?;

    response.text().await.unexpected_err()
}
