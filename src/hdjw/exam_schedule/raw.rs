use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    hdjw::{error::TokenExpired, login::HdjwToken, raw::HdjwResponseExtractor},
    utils::client,
};
use serde::Deserialize;

// 该 URL 缺少学期的参数，需要后续再用 format 拼接
const EXAM_SCHEDULE_URL: &str =
    "http://hdjw.hnu.edu.cn/jsxsd/xsks/xsksap_list?pageNum=1&pageSize=20&xqlb=";

/// 考试安排单项
/// 带 Option 的字段应该是类似于体育理论这样考试安排信息很不全的课程
#[derive(Deserialize, Debug)]
pub struct ExamScheduleItem {
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kskcmc: String,
    /// 考试校区
    pub ksxq: Option<String>,
    /// 考试的教室
    pub js_mc: Option<String>,
    /// 考试时间（已经是一个时间区间了）
    pub kssj: Option<String>,
    /// 座位号
    pub zwh: Option<String>,
}

pub async fn raw_exam_schedule_data(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<ExamScheduleItem>, crate::Error<TokenExpired>> {
    let headers = hdjw_token.headers().clone();
    let res = client
        .get(format!(
            "{}&xnxqid={}-{}-{}",
            EXAM_SCHEDULE_URL,
            xn,
            xn + 1,
            xq
        ))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data()
        .await?;
    let res: Vec<ExamScheduleItem> =
        serde_json::from_value(res["data"].clone()).parse_err(&res.to_string())?;
    Ok(res)
}
