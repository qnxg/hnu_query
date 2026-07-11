use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
    yjsxt::{error::TokenExpired, fetch::YjsxtResponse, login::YjsxtToken},
};

const GRADUATE_HOST_URL: &str = "http://yjsxt.hnu.edu.cn/gmis/";
const BIND_TERM_URL: &str = "/student/default/bindterm";

pub async fn semester(yjsxt_token: &YjsxtToken) -> Result<String, crate::Error<TokenExpired>> {
    let url = format!("{}{}{}", GRADUATE_HOST_URL, yjsxt_token.id(), BIND_TERM_URL);
    client
        .get(&url)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .check_token_expired()?
        .text()
        .await
        .unexpected_err()
}
