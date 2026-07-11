use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    pt::login::PtToken,
    utils::client,
};
use std::convert::Infallible;

const CARD_INFO_URL: &str = "https://pt.hnu.edu.cn/api/hndxYkt/getCardUserInfo/info";
const CSRF_TOKEN_URL: &str = "https://pt.hnu.edu.cn/api/security/token";
const CARD_HISTORY_URL: &str = "https://pt.hnu.edu.cn/api/hndxYkt/getAccHisConsubDzzfLog/detail";

pub async fn csrf_token(pt_token: &PtToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(CSRF_TOKEN_URL)
        .headers(pt_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}

pub async fn card_info(pt_token: &PtToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(CARD_INFO_URL)
        .headers(pt_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}

/// `csrf_token` 为 [super::parse::csrf_token] 的返回数据
pub async fn card_history(
    pt_token: &PtToken,
    csrf_token: &str,
    year: u16,
    month: u8,
    trancode: &str,
) -> Result<String, crate::Error<Infallible>> {
    // 字符串格式化默认是左对齐，这里要手动改成右对齐，并且两位宽左侧补0
    let begin_date = format!("{}-{:0>2}-01", year, month);
    // 这里没有必要精确查询日历好像是？直接取31号
    let end_date = format!("{}-{:0>2}-31", year, month);

    let form_data = [
        ("beginDate", begin_date.as_str()),
        ("endDate", end_date.as_str()),
        ("pageSize", "100000"),
        ("trancode", trancode),
    ];
    client
        .post(CARD_HISTORY_URL)
        .headers(pt_token.headers().clone())
        .header("X-XSRF-TOKEN", csrf_token)
        .form(&form_data)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}
