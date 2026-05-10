mod raw;

use crate::{
    error::{MapParseErr, parse_err, parse_err_with_reason},
    yjsxt::{error::TokenExpired, login::YjsxtToken},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// 研究生课程信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Course {
    /// 课程名称
    pub course_name: String,
    /// 课程代码
    pub course_id: String,
    /// 上课班级
    pub class_name: String,
    /// 授课教师
    pub teacher: Option<String>,
    /// 课程的时间地点安排
    ///
    /// 如果该课程是无节次课程，则为 None
    pub schedule: Option<Vec<CourseSchedule>>,
}

/// 研究生课程时间地点安排
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CourseSchedule {
    /// 第几周上课
    pub week: u8,
    /// 周几上课 (1=周一, 7=周日)
    pub day: u8,
    /// 上课地点
    pub place: String,
    /// 上课节次
    pub time: Vec<u8>,
}

#[derive(Hash, Eq, PartialEq)]
struct ParsedCourse {
    pub course_name: String,
    pub course_id: String,
    pub class_name: String,
    pub teacher: Option<String>,
}

fn parse_course_info(
    cell_text: &str,
) -> Result<(ParsedCourse, HashSet<u8>, String), crate::Error<TokenExpired>> {
    let class_time_regex =
        Regex::new(r"上课时间:.*\[([0-9\-]+)周\].*连续周").expect("创建正则表达式失败");
    let teacher_and_classroom_regex = Regex::new(r"(.*)\[(.*)\]").expect("创建正则表达式失败");

    let parts: Vec<&str> = cell_text.split("<br/>").filter(|s| !s.is_empty()).collect();

    // 研究生系统返回的信息可能有多余的神秘空格，要去掉
    let course_id = parts
        .first()
        .ok_or(parse_err(cell_text))?
        .replace("课程编号:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let course_name = parts
        .get(1)
        .ok_or(parse_err(cell_text))?
        .replace("课程名称:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let class_name = parts
        .get(2)
        .ok_or(parse_err(cell_text))?
        .replace("班级:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    // 上课时间: [9-16周] 连续周
    let class_time_str = parts
        .get(3)
        .ok_or(parse_err(cell_text))?
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let class_time = class_time_regex
        .captures(&class_time_str)
        .and_then(|c| c.get(1))
        .and_then(|c| {
            c.as_str()
                .split('-')
                .map(|s| s.parse::<u8>().ok())
                .collect::<Option<Vec<_>>>()
        })
        .ok_or(parse_err_with_reason(&class_time_str, "解析上课时间失败"))?;
    let Some(weeks_l) = class_time.first() else {
        return Err(parse_err_with_reason(&class_time_str, "解析上课时间失败"));
    };
    // 可能只有一个周次
    let weeks_r = class_time.get(1).unwrap_or(weeks_l);

    let teacher_and_classroom_str = parts
        .get(4)
        .ok_or(parse_err(cell_text))?
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let Some(teacher_and_classroom) = teacher_and_classroom_regex
        .captures(&teacher_and_classroom_str)
        .and_then(|c| {
            c.iter()
                .map(|c| c.map(|v| v.as_str().to_string()))
                .collect::<Option<Vec<_>>>()
        })
    else {
        return Err(parse_err_with_reason(
            &teacher_and_classroom_str,
            "解析授课老师和上课地点失败",
        ));
    };
    let [_, teacher, classroom] = teacher_and_classroom.try_into().map_err(|_| {
        parse_err_with_reason(&teacher_and_classroom_str, "解析授课老师和上课地点失败")
    })?;

    let res = ParsedCourse {
        course_name,
        course_id,
        class_name,
        teacher: if teacher.is_empty() {
            None
        } else {
            Some(teacher)
        },
    };
    Ok((res, (*weeks_l..=*weeks_r).collect(), classroom))
}

/// 获取课表信息
///
/// # Arguments
///
/// * `yjsxt_token` - 研究生系统的令牌，可以通过 [YjsxtToken::acquire_by_cas_login] 获取
/// * `termcode` - 学期代码，可以通过 [get_termcode](super::get_termcode) 获取
///
/// # Returns
///
/// 返回所选课程的列表
///
/// # Errors
///
/// 如果提供的 `yjsxt_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [YjsxtToken]
pub async fn get_class_table(
    yjsxt_token: &YjsxtToken,
    termcode: u16,
) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let raw_rows = raw::raw_class_table_data(yjsxt_token, termcode).await?;

    // 研究生系统的课表的颗粒度比我们的更细，他们把节次信息也拆掉了
    // 所以我们这里需要把同一个课程，同一周次、周几、上课地点的节次信息合并
    // (week, day, place)
    type UniqueScheduleKey = (u8, u8, String);
    let mut course_map: HashMap<ParsedCourse, HashMap<UniqueScheduleKey, Vec<u8>>> = HashMap::new();
    // 防止有多周的无课表课程的出现，此时研究生系统可能有重复，所以这里对无课表课程去重
    let mut extra_courses: HashMap<ParsedCourse, ()> = HashMap::new();

    for item in raw_rows {
        let jc = item["mc"]
            .as_str()
            .ok_or(parse_err_with_reason(&item.to_string(), "解析节次失败"))?;
        for day in 1..=7u8 {
            let key = format!("z{day}");
            if item[&key] == Value::Null {
                continue;
            }

            let cell_text = item[&key].as_str().ok_or(parse_err(&item.to_string()))?;
            let (course_info, weeks, place) = parse_course_info(cell_text)?;

            if jc == "无节次" {
                extra_courses.insert(course_info, ());
            } else {
                let jc = jc.parse::<u8>().parse_err_with_reason(jc, "解析节次失败")?;
                let entry = course_map.entry(course_info).or_default();
                for week in weeks {
                    let entry = entry.entry((week, day, place.clone())).or_default();
                    entry.push(jc);
                }
            }
        }
    }

    let mut res = Vec::new();

    for (course_info, schedule) in course_map {
        let course_schedule = schedule
            .into_iter()
            .map(|((week, day, place), time)| CourseSchedule {
                week,
                day,
                place,
                time: time.into_iter().collect(),
            })
            .collect();
        res.push(Course {
            course_name: course_info.course_name,
            course_id: course_info.course_id,
            class_name: course_info.class_name,
            teacher: course_info.teacher,
            schedule: Some(course_schedule),
        });
    }

    for (course_info, _) in extra_courses {
        res.push(Course {
            course_name: course_info.course_name,
            course_id: course_info.course_id,
            class_name: course_info.class_name,
            teacher: course_info.teacher,
            schedule: None,
        });
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::{
        test::{TEST_XN, TEST_XQ},
        yjsxt::test::get_yjsxt_token,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_get_class_table() {
        let yjsxt_token = get_yjsxt_token().await.unwrap();
        let termcode = crate::yjsxt::get_termcode(&yjsxt_token, *TEST_XN, *TEST_XQ)
            .await
            .unwrap();
        let class_table = get_class_table(&yjsxt_token, termcode).await.unwrap();
        println!("{:#?}", class_table);
    }
}
