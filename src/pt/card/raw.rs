use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    pt::login::PtToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

const CARD_INFO_URL: &str = "https://pt.hnu.edu.cn/api/hndxYkt/getCardUserInfo/info";
const CSRF_TOKEN_URL: &str = "https://pt.hnu.edu.cn/api/security/token";
const CARD_HISTORY_URL: &str = "https://pt.hnu.edu.cn/api/hndxYkt/getAccHisConsubDzzfLog/detail";

#[derive(Deserialize, Debug)]
pub struct RawCardInfo {
    pub account: u32,
    pub balance: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct RawCardHistory {
    pub amt: f64,
    pub count: f64,
    pub webTrjnDTO: Option<Vec<RawCardHistoryItem>>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct RawCardHistoryItem {
    pub fTranAmt: String,
    pub jndatetime: String,
    pub effectdate: String,
    pub jourName: String,
    pub usedcardnum: u32,
    pub nowAmt: String,
    pub sysname1: Option<String>,
    pub tranname: String,
}

pub async fn get_card_user_info(
    pt_token: &PtToken,
) -> Result<RawCardInfo, crate::Error<Infallible>> {
    let json_str = client
        .get(CARD_INFO_URL)
        .headers(pt_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;

    let data = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(data)
}

pub async fn get_acc_history(
    pt_token: &PtToken,
    year: u16,
    month: u8,
    trancode: &str,
) -> Result<RawCardHistory, crate::Error<Infallible>> {
    let headers = pt_token.headers().clone();
    // 字符串格式化默认是左对齐，这里要手动改成右对齐，并且两位宽左侧补0
    let begin_date = format!("{}-{:0>2}-01", year, month);
    // 这里没有必要精确查询日历好像是？直接取31号
    let end_date = format!("{}-{:0>2}-31", year, month);

    let csrf_json_str = client
        .get(CSRF_TOKEN_URL)
        .headers(headers.clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let csrf_token = serde_json::from_str::<Value>(&csrf_json_str)
        .parse_err(&csrf_json_str)?
        .get("data")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or(parse_err(&csrf_json_str))?;

    let form_data = [
        ("beginDate", begin_date.as_str()),
        ("endDate", end_date.as_str()),
        ("pageSize", "100000"),
        ("trancode", trancode),
    ];
    let json_str = client
        .post(CARD_HISTORY_URL)
        .headers(pt_token.headers().clone())
        .header("X-XSRF-TOKEN", &csrf_token)
        .form(&form_data)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;

    let data = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(data)
}
