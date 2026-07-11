mod fetch;
mod parse;

use crate::hdjw::{error::TokenExpired, login::HdjwToken};
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
    let json_str = fetch::exam_schedule(hdjw_token, xn, xq).await?;
    parse::exam_schedule(&json_str)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hdjw::test::get_hdjw_token;
    use crate::test::{TEST_XN, TEST_XQ, TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_exam_schedule() -> TestResult<()> {
        let hdjw_token = get_hdjw_token().await?;
        let exam_schedule = get_exam_schedule(&hdjw_token, *TEST_XN, *TEST_XQ).await?;
        println!("{:#?}", exam_schedule);
        Ok(())
    }
}
