mod fetch;
mod parse;

use crate::lab::login::LabToken;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 大物实验安排
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct LabSchedule {
    /// 座位号
    pub seat: String,
    /// 实验名称
    pub name: String,
    /// 实验所属的课程名称
    pub course: String,
    /// 授课老师
    pub teacher: String,
    /// 时间周次
    pub week: u8,
    /// 星期几
    ///
    /// 星期一为 `1`，星期日为 `7`
    pub day: u8,
    /// 实验的日期和时间
    pub date_time: NaiveDateTime,
    /// 实验地点
    pub place: String,
    /// 授课老师的联系电话
    pub phone: Option<String>,
    /// 授课老师的邮箱
    pub email: Option<String>,
}

/// 获取大物实验安排
///
/// # Arguments
///
/// - `lab_token`: 大物实验平台的令牌，可以通过 [LabToken::acquire_by_login] 获取
///
/// # Returns
///
/// 返回一个包含所有大物实验安排的列表
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(lab_token), fields(subsystem = "lab"), err)
)]
pub async fn get_lab_schedule(
    lab_token: &LabToken,
) -> Result<Vec<LabSchedule>, crate::Error<Infallible>> {
    let json_str = fetch::lab_schedule(lab_token).await?;
    parse::lab_schedule(&json_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lab::test::get_lab_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_lab_schedule() -> TestResult<()> {
        let lab_token = get_lab_token().await?;
        let schedule = get_lab_schedule(&lab_token).await?;
        println!("{:#?}", schedule);
        Ok(())
    }
}
