use crate::{
    error::MapUnexpectedErr,
    iportal::{
        error::IPortalTokenExpired, login::IPortalToken, personal::PersonalDataTypeEnum,
        util::IPortalRequestBuilderExt,
    },
    utils::client,
};

pub async fn fetch_personal_data_query_ids(
    token: &IPortalToken,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    client
        .get("https://iportal.hnu.edu.cn/personal/frontend/data/items?type=personal_data")
        .send_with_token(token)
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn fetch_personal_data(
    token: &IPortalToken,
    type_enum: PersonalDataTypeEnum,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    client
        .get(format!(
            "https://iportal.hnu.edu.cn/personal/frontend/data/detail?id={}",
            type_enum.into_value()
        ))
        .send_with_token(token)
        .await?
        .text()
        .await
        .unexpected_err()
}
