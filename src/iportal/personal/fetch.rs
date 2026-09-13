use crate::{
    cas::error::TokenExpired,
    error::MapUnexpectedErr,
    iportal::{
        login::IPortalToken, personal::PersonalDataTypeEnum, util::IPortalRequestBuilderExt,
    },
    utils::client,
};

pub async fn fetch_personal_data_query_ids(
    token: &IPortalToken,
) -> Result<String, crate::Error<TokenExpired>> {
    let url = "https://iportal.hnu.edu.cn/personal/frontend/data/items?type=personal_data";
    let response = client.get(url).send_with_token(token).await?;

    response.text().await.unexpected_err()
}

pub async fn fetch_personal_data(
    token: &IPortalToken,
    type_enum: PersonalDataTypeEnum,
) -> Result<String, crate::Error<TokenExpired>> {
    let url = format!(
        "https://iportal.hnu.edu.cn/personal/frontend/data/detail?id={}",
        type_enum.into_value()
    );
    let response = client.get(&url).send_with_token(token).await?;

    response.text().await.unexpected_err()
}
