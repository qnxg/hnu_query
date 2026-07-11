use crate::{
    error::{MapParseErr, parse_err, parse_err_with_reason},
    lab::schedule::LabSchedule,
};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawLabSchedule {
    /// 座位号
    SeatNo: String,
    /// 实验名称
    LabName: String,
    /// 课程名称
    CourseName: String,
    /// 上课老师名称
    UserName: String,
    /// 上课周次
    Weeks: String,
    /// 上课星期几
    WeekName: String,
    /// 上课日期，格式如“2025/9/27 0:00:00”目前来看就前面的日期部分正确
    ClassDate: String,
    /// 上课开始时间
    StartTime: String,
    /// 上课地点
    ClassRoom: String,
    /// 联系电话
    MobileNum: String,
    /// 联系邮箱
    Email: String,
}

pub fn lab_schedule(json_str: &str) -> Result<Vec<LabSchedule>, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("rows")
        .map(|v| serde_json::from_value::<Vec<RawLabSchedule>>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;
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
            .ok_or_else(|| parse_err_with_reason(&item.ClassDate, "date"))?;
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
