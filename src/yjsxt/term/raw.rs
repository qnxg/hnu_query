use serde::Deserialize;

use crate::{
    error::MapNetworkErr,
    utils::client,
    yjsxt::{error::TokenExpired, login::YjsxtToken, utils::YjsxtResponseExtractor},
};

const GRADUATE_HOST_URL: &str = "http://yjsxt.hnu.edu.cn/gmis/";
const BIND_TERM_URL: &str = "/student/default/bindterm";

/// API 返回的学期信息
#[derive(Deserialize, Debug)]
pub struct TermItem {
    pub termcode: String,
    pub termname: String,
}

pub async fn raw_termcode(
    yjsxt_token: &YjsxtToken,
) -> Result<Vec<TermItem>, crate::Error<TokenExpired>> {
    let url = format!("{}{}{}", GRADUATE_HOST_URL, yjsxt_token.id(), BIND_TERM_URL);
    let terms = client
        .get(&url)
        .send()
        .await
        .network_err()?
        .extract_data(true)
        .await?;
    Ok(terms)
}
