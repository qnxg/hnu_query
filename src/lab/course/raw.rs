use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    lab::login::LabToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, convert::Infallible};

const COURSE_LIST_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentScoreList";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct CourseItem {
    /// 课程名称
    pub CourseName: String,
    /// 课程总成绩，没有成绩的话是空字符串
    ///
    /// 如果需要获取课程的具体成绩，请使用 `lab::get_lab_grade` 来获取
    pub CourseFinalScore: String,
    /// 课程id
    pub CourseID: String,
}

pub async fn raw_course_list_data(
    lab_token: &LabToken,
    semester_id: &str,
) -> Result<Vec<CourseItem>, crate::Error<Infallible>> {
    let headers = lab_token.headers().clone();
    let mut form_data = HashMap::new();
    form_data.insert("page", "1");
    form_data.insert("rows", "15");
    form_data.insert("SemID", semester_id);
    form_data.insert("UserID", lab_token.stu_id());
    let json_str = client
        .post(COURSE_LIST_URL)
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
        .map(|v| serde_json::from_value::<Vec<CourseItem>>(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}
