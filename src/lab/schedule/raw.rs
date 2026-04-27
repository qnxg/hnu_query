use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    lab::login::LabToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, convert::Infallible};

const LAB_LIST_URL: &str = "http://10.62.106.112/XPK/StuCourseElectiveLook/LoadTableInfo";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScheduleItem {
    /// 座位号
    pub SeatNo: String,
    /// 实验名称
    pub LabName: String,
    /// 课程名称
    pub CourseName: String,
    /// 上课老师名称
    pub UserName: String,
    /// 上课周次
    pub Weeks: String,
    /// 上课星期几
    pub WeekName: String,
    /// 上课日期，格式如“2025/9/27 0:00:00”目前来看就前面的日期部分正确
    pub ClassDate: String,
    /// 上课开始时间
    pub StartTime: String,
    /// 上课地点
    pub ClassRoom: String,
    /// 联系电话
    pub MobileNum: String,
    /// 联系邮箱
    pub Email: String,
}

pub async fn raw_lab_schedule_data(
    lab_token: &LabToken,
) -> Result<Vec<LabScheduleItem>, crate::Error<Infallible>> {
    let headers = lab_token.headers().clone();
    let mut form_data = HashMap::new();
    form_data.insert("CourseID", "-999");
    form_data.insert("weeks", "-999");
    form_data.insert("labID", "-999");
    form_data.insert("page", "1");
    form_data.insert("rows", "200");
    let json_str = client
        .post(LAB_LIST_URL)
        .form(&form_data)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("rows")
        .map(|v| serde_json::from_value::<Vec<LabScheduleItem>>(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}
