use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    hdjw::{error::TokenExpired, login::HdjwToken},
    utils::client,
};

const GRADE_RANK_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xscjsq/cjpmcx_list.do";

pub async fn rank(
    hdjw_token: &HdjwToken,
    selection: &str,
    range: &str,
    data_source: &str,
    display: &str,
) -> Result<String, crate::Error<TokenExpired>> {
    let form_data = [
        ("xnxq01id", selection),
        ("kkxz1", ""),
        ("pmfs1", ""),
        ("kclx1", range),       // 方案类别
        ("kcly1", data_source), // 数据来源
        ("xsfs1", display),     // 显示方式
    ];
    client
        .post(GRADE_RANK_URL)
        .form(&form_data)
        .headers(hdjw_token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
