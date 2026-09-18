use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    iportal::{
        error::{CheckIPortalTokenExpired, IPortalTokenExpired},
        login::IPortalToken,
        personal::PersonalDataTypeEnum,
    },
    utils::client,
};

pub async fn personal_data_query_ids(
    token: &IPortalToken,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    let url = "https://iportal.hnu.edu.cn/personal/frontend/data/items?type=personal_data";
    client
        .get(url)
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .iportal_token_expired()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn personal_data(
    token: &IPortalToken,
    type_enum: PersonalDataTypeEnum,
) -> Result<String, crate::Error<IPortalTokenExpired>> {
    let url = format!(
        "https://iportal.hnu.edu.cn/personal/frontend/data/detail?id={}",
        type_enum.get_value()
    );
    client
        .get(url)
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
