use crate::{
    error::{MapNetworkErr, MapUnexpectedErr, parse_err},
    hdjw::{error::TokenExpired, login::HdjwToken, raw::HdjwResponseExtractor},
    utils::client,
};
use serde_json::Value;

const GRADE_RANK_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xscjsq/cjpmcx_list.do";

pub async fn raw_rank_data(
    hdjw_token: &HdjwToken,
    selection: &str,
    range: &str,
    data_source: &str,
    display: &str,
) -> Result<Value, crate::Error<TokenExpired>> {
    let form_data = [
        ("xnxq01id", selection),
        ("kkxz1", ""),
        ("pmfs1", ""),
        ("kclx1", range),       // 方案类别
        ("kcly1", data_source), // 数据来源
        ("xsfs1", display),     // 显示方式
    ];
    let headers = hdjw_token.headers().clone();
    let res = client
        .post(GRADE_RANK_URL)
        .form(&form_data)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data()
        .await?;
    match res.get("data") {
        Some(data @ Value::Object(_)) => Ok(data.clone()),
        _ => Err(parse_err(&res.to_string())),
    }
}
