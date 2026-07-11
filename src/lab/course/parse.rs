use crate::{
    error::{MapParseErr, parse_err},
    lab::course::Course,
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawCourse {
    /// 课程名称
    CourseName: String,
    /// 课程总成绩，没有成绩的话是空字符串
    ///
    /// 如果需要获取课程的具体成绩，请使用 `lab::get_lab_grade` 来获取
    CourseFinalScore: String,
    /// 课程id
    CourseID: String,
}

/// `json_str` 为 [`super::fetch::raw_course_list_data`] 返回的数据
pub fn course_list(json_str: &str) -> Result<Vec<Course>, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("rows")
        .map(|v| serde_json::from_value::<Vec<RawCourse>>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        res.push(Course {
            name: item.CourseName,
            score: if item.CourseFinalScore.is_empty() {
                None
            } else {
                Some(item.CourseFinalScore)
            },
            id: item.CourseID,
        });
    }
    Ok(res)
}
