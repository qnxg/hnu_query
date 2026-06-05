use serde_json::Value;

use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
    xgxt::login::XgxtToken,
};
use std::convert::Infallible;

// 个人信息
const USER_INFO_URL: &str =
    "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/userInfo/default.zf?dataId=null";
// 在校信息
const IN_SCHOOL_INFO_URL: &str =
    "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/zxxx/default.zf?dataId=null";
// 联系方式
const CONTACT_INFO_URL: &str =
    "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/lxfs1/default.zf?dataId=null";

async fn raw_with_url(
    xgxt_token: &XgxtToken,
    url: &str,
) -> Result<Value, crate::Error<Infallible>> {
    client
        .get(url)
        .headers(xgxt_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .json()
        .await
        .unexpected_err()
}

pub async fn raw_user_info(xgxt_token: &XgxtToken) -> Result<Value, crate::Error<Infallible>> {
    raw_with_url(xgxt_token, USER_INFO_URL).await
}

pub async fn raw_in_school_info(xgxt_token: &XgxtToken) -> Result<Value, crate::Error<Infallible>> {
    raw_with_url(xgxt_token, IN_SCHOOL_INFO_URL).await
}

pub async fn raw_contact_info(xgxt_token: &XgxtToken) -> Result<Value, crate::Error<Infallible>> {
    raw_with_url(xgxt_token, CONTACT_INFO_URL).await
}
