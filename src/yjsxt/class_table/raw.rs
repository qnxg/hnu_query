use serde_json::Value;

use crate::{
    error::{MapNetworkErr, parse_err},
    utils::client,
    yjsxt::{error::TokenExpired, login::YjsxtToken, utils::YjsxtResponseExtractor},
};

const GRADUATE_HOST_URL: &str = "http://yjsxt.hnu.edu.cn/gmis/";
const CLASS_TABLE_URL: &str = "/student/pygl/py_kbcx_ew";

pub async fn raw_class_table_data(
    yjsxt_token: &YjsxtToken,
    termcode: u16,
) -> Result<Vec<Value>, crate::Error<TokenExpired>> {
    let url = format!(
        "{}{}{}",
        GRADUATE_HOST_URL,
        yjsxt_token.id(),
        CLASS_TABLE_URL
    );
    let res: Value = client
        .post(&url)
        .form(&[("kblx", "xs"), ("termcode", &termcode.to_string())])
        .send()
        .await
        .network_err()?
        .extract_data(true)
        .await?;

    let rows = res["rows"]
        .as_array()
        .ok_or(parse_err(&serde_json::to_string(&res).unwrap_or_default()))?;

    Ok(rows.clone())
}
