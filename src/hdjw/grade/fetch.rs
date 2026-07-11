use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    hdjw::{error::TokenExpired, login::HdjwToken},
    utils::client,
};

// 课程成绩查询接口
// 该 URL 缺少学期的参数，需要后续再用 format 拼接
const GRADE_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/kscj/cjcx_list?pageNum=1&pageSize=50&kcxz=&kcsx=&kcmc=&xsfs=all&sfxsbcxq=1";

// 该 URL 缺少 jx0404id 的参数，需要后续再用 format 拼接
const GRADE_DETAIL_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/kscj/pscj_list.do?zcj=";

pub async fn get_cjcx_list(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<String, crate::Error<TokenExpired>> {
    client
        .get(format!("{}&kksj={}-{}-{}", GRADE_URL, xn, xn + 1, xq))
        .headers(hdjw_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}

// 返回的原始数据是 html 格式
pub async fn get_pscj_list(
    hdjw_token: &HdjwToken,
    jx0404id: &str,
) -> Result<String, crate::Error<TokenExpired>> {
    client
        .get(format!("{}&jx0404id={}", GRADE_DETAIL_URL, jx0404id))
        .headers(hdjw_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}
