use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
    yjsxt::{error::TokenExpired, fetch::YjsxtResponse, login::YjsxtToken},
};

const GRADUATE_HOST_URL: &str = "http://yjsxt.hnu.edu.cn/gmis/";
const CLASS_TABLE_URL: &str = "/student/pygl/py_kbcx_ew";

pub async fn class_table(
    yjsxt_token: &YjsxtToken,
    semester_id: u16,
) -> Result<String, crate::Error<TokenExpired>> {
    let url = format!(
        "{}{}{}",
        GRADUATE_HOST_URL,
        yjsxt_token.id(),
        CLASS_TABLE_URL
    );
    client
        .post(&url)
        .form(&[("kblx", "xs"), ("termcode", &semester_id.to_string())])
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
