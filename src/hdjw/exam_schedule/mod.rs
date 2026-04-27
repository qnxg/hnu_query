mod raw;

use crate::{
    error::{MapParseErr, parse_err},
    hdjw::{error::TokenExpired, exam_schedule::raw::raw_exam_schedule_data, login::HdjwToken},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 考试安排
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ExamSchedule {
    /// 考试课程的课程代码
    pub course_id: String,
    /// 考试课程的课程名称
    pub course_name: String,
    /// 考试校区
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    pub area: Option<String>,
    /// 考试的教室
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    pub classroom: Option<String>,
    /// 考试的日期
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    ///
    /// `date` 和 `time` 会同时为 `None` 或同时为 `Some`
    pub date: Option<NaiveDate>,
    /// 考试的时间，为一个时间段，如 `14:00~16:00`
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    ///
    /// `date` 和 `time` 会同时为 `None` 或同时为 `Some`
    pub time: Option<String>,
    /// 考试的座位号
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    pub seat: Option<String>,
}

/// 获取考试安排
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌，可以通过 [HdjwToken::acquire_by_cas_login] 获取
/// - `xn`: 学年
/// - `xq`: 学期
///
/// # Returns
///
/// 返回一个包含给定学年学期的考试安排的列表
///
/// # Errors
///
/// 如果提供的 `hdjw_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [HdjwToken]
pub async fn get_exam_schedule(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<ExamSchedule>, crate::Error<TokenExpired>> {
    let raw_data = raw_exam_schedule_data(hdjw_token, xn, xq).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let (date, time) = match item.kssj {
            Some(kssj) => {
                let [date, time] = kssj.split(' ').collect::<Vec<_>>()[..] else {
                    return Err(parse_err(&kssj));
                };
                let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").parse_err(date)?;
                (Some(date), Some(time.to_string()))
            }
            None => (None, None),
        };

        res.push(ExamSchedule {
            course_id: item.kch,
            course_name: item.kskcmc,
            area: item.ksxq,
            classroom: item.js_mc,
            date,
            time: time.map(|s| s.to_string()),
            seat: item.zwh,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hdjw::test::get_hdjw_token;
    use crate::test::{TEST_XN, TEST_XQ};

    #[tokio::test]
    #[ignore]
    async fn test_get_exam_schedule() {
        let hdjw_token = get_hdjw_token().await.unwrap();
        let exam_schedule = get_exam_schedule(&hdjw_token, *TEST_XN, *TEST_XQ)
            .await
            .unwrap();
        println!("{:#?}", exam_schedule);
    }
}
