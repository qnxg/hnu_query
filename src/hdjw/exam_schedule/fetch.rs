use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    hdjw::{error::TokenExpired, login::HdjwToken},
    utils::client,
};

// 该 URL 缺少学期的参数，需要后续再用 format 拼接
const EXAM_SCHEDULE_URL: &str =
    "http://hdjw.hnu.edu.cn/jsxsd/xsks/xsksap_list?pageNum=1&pageSize=20&xqlb=";

pub async fn exam_schedule(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<String, crate::Error<TokenExpired>> {
    client
        .get(format!(
            "{}&xnxqid={}-{}-{}",
            EXAM_SCHEDULE_URL,
            xn,
            xn + 1,
            xq
        ))
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
