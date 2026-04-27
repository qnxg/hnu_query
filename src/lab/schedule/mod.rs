mod raw;

use crate::{
    error::{MapParseErr, parse_err_with_reason},
    lab::login::LabToken,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
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
pub async fn get_lab_schedule(
    lab_token: &LabToken,
) -> Result<Vec<LabSchedule>, crate::Error<Infallible>> {
    let raw_data = raw::raw_lab_schedule_data(lab_token).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let day = match item.WeekName.as_str() {
            "星期一" => 1,
            "星期二" => 2,
            "星期三" => 3,
            "星期四" => 4,
            "星期五" => 5,
            "星期六" => 6,
            "星期日" => 7,
            _ => {
                return Err(parse_err_with_reason(&item.WeekName, "day"));
            }
        };
        let week = item
            .Weeks
            .parse::<u8>()
            .parse_err_with_reason(&item.Weeks, "week")?;
        let date = item
            .ClassDate
            .split(' ')
            .next()
            .map(|v| NaiveDate::parse_from_str(v, "%Y/%m/%d").parse_err_with_reason(v, "date"))
            .transpose()?
            .ok_or(parse_err_with_reason(&item.ClassDate, "date"))?;
        let time = NaiveTime::parse_from_str(&item.StartTime, "%H:%M")
            .parse_err_with_reason(&item.StartTime, "time")?;
        let tmp = LabSchedule {
            seat: item.SeatNo,
            name: item.LabName,
            course: item.CourseName,
            teacher: item.UserName,
            week,
            day,
            date_time: date.and_time(time),
            place: item.ClassRoom,
            phone: if item.MobileNum.is_empty() {
                None
            } else {
                Some(item.MobileNum)
            },
            email: if item.Email.is_empty() {
                None
            } else {
                Some(item.Email)
            },
        };
        res.push(tmp);
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lab::test::get_lab_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_lab_schedule() {
        let lab_token = get_lab_token().await.unwrap();
        let schedule = get_lab_schedule(&lab_token).await.unwrap();
        println!("{:#?}", schedule);
    }
}
