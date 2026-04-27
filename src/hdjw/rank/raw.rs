use crate::{
    error::{MapNetworkErr, MapUnexpectedErr, parse_err},
    hdjw::{error::TokenExpired, login::HdjwToken, raw::HdjwResponseExtractor},
    utils::client,
};
use serde_json::Value;

// 该 URL 缺少学期、课程类型，排名方式的参数，需要后续再用 format 拼接
const GRADE_RANK_URL: &str =
    "http://hdjw.hnu.edu.cn/jsxsd/xscjsq/cjpmcx_list.do?&pageNum=1&pageSize=20&kclx=&kcly=1";

pub async fn raw_rank_data(
    hdjw_token: &HdjwToken,
    selection: &str,
    range: &str,
    rank_method: &str,
) -> Result<Option<Value>, crate::Error<TokenExpired>> {
    let headers = hdjw_token.headers().clone();
    let res = client
        .get(format!(
            "{}&xnxq={}&kkxz={}&pmfs={}",
            GRADE_RANK_URL, selection, range, rank_method
        ))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data()
        .await?;
    let Some(data) = res["data"].as_array() else {
        return Err(parse_err(&res.to_string()));
    };
    Ok(data.first().cloned())
}
