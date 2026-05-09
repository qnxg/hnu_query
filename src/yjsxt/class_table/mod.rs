mod raw;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{MapParseErr, parse_err},
    yjsxt::{error::TokenExpired, login::YjsxtToken},
};

/// 研究生课程信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraduateCourse {
    /// 课程名称
    pub course_name: String,
    /// 课程代码
    pub course_id: String,
    /// 上课班级
    pub class_name: String,
    /// 授课教师
    pub teacher: Option<String>,
    /// 课程的时间地点安排
    pub schedule: Vec<GraduateCourseSchedule>,
}

/// 研究生课程时间地点安排
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraduateCourseSchedule {
    /// 第几周上课
    pub week: u8,
    /// 周几上课 (1=周一, 7=周日)
    pub day: u8,
    /// 上课地点
    pub place: String,
    /// 上课节次
    pub time: Vec<u8>,
}

struct CourseInfo {
    course_id: String,
    course_name: String,
    class_name: String,
    class_time: String,
    teacher: String,
    classroom: String,
}

fn parse_course_info(input: &str) -> Option<CourseInfo> {
    let parts: Vec<&str> = input.split("<br/>").filter(|s| !s.is_empty()).collect();

    let course_id = parts[0].replace("课程编号:", "").trim().to_string();
    let course_name = parts[1].replace("课程名称:", "").trim().to_string();
    let class_name = parts[2].replace("班级:", "").trim().to_string();
    let class_time = parts
        .iter()
        .find(|s| s.contains("上课时间:"))
        .map(|s| s.replace("上课时间:", "").trim().to_string())?;
    // 原格式: [9-16周] 连续周 → 生成 "9,10,11,...,16"
    let class_time = class_time
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-')
        .collect::<String>();
    let class_time = class_time.split('-').collect::<Vec<&str>>();
    let class_time = (class_time[0].parse::<u8>().ok()?..=class_time[1].parse::<u8>().ok()?)
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let teacher_and_classroom = parts
        .iter()
        .find(|s| s.contains('[') && !s.starts_with("上课时间:"))
        .map(|s| s.trim())
        .unwrap_or("[]");

    let teacher_end = teacher_and_classroom.chars().position(|c| c == '[');
    let (teacher, classroom) = match teacher_end {
        Some(end) => {
            let t: String = teacher_and_classroom.chars().take(end).collect();
            let classroom_start = end + 1;
            let classroom_end = teacher_and_classroom
                .chars()
                .position(|c| c == ']')
                .unwrap_or(teacher_and_classroom.len());
            let c: String = teacher_and_classroom
                .chars()
                .skip(classroom_start)
                .take(classroom_end - classroom_start)
                .collect();
            (t, c)
        }
        None => (String::new(), String::new()),
    };

    Some(CourseInfo {
        course_id,
        course_name,
        class_name,
        class_time,
        teacher,
        classroom,
    })
}

type CourseKey = (String, String, String, String);
type RawScheduleEntry = (u8, u8, String, u8);

fn build_graduate_course_info(
    rows: &[Value],
) -> Result<Vec<GraduateCourse>, crate::Error<TokenExpired>> {
    // key = (course_id, course_name, class_name, teacher)
    // value = Vec<(week, day, place, section)>
    let mut course_map: HashMap<CourseKey, Vec<RawScheduleEntry>> = HashMap::new();

    for item in rows {
        if item["mc"] == Value::String("无节次".to_string()) {
            continue;
        }
        let jc = item["mc"]
            .as_str()
            .ok_or(parse_err(&serde_json::to_string(&item).unwrap_or_default()))?
            .parse::<u8>()
            .parse_err_with_reason("", "解析节次失败")?;

        for day in 1..=7u8 {
            let key = format!("z{day}");
            if item[&key] == Value::Null {
                continue;
            }
            let cell_text = item[&key].as_str().ok_or(parse_err(&item.to_string()))?;
            let course_info = parse_course_info(cell_text).ok_or(parse_err(cell_text))?;

            let weeks: Vec<u8> = course_info
                .class_time
                .split(',')
                .filter_map(|s| s.parse().ok())
                .collect();

            let key = (
                course_info.course_id,
                course_info.course_name,
                course_info.class_name,
                course_info.teacher,
            );
            let entry = course_map.entry(key).or_default();
            for week in weeks {
                entry.push((week, day, course_info.classroom.clone(), jc));
            }
        }
    }

    let mut courses = Vec::with_capacity(course_map.len());
    for ((course_id, course_name, class_name, teacher), raw_schedule) in course_map {
        // 按 (week, day, place) 分组，收集节次
        let mut schedule_map: HashMap<(u8, u8, String), HashSet<u8>> = HashMap::new();
        for (week, day, place, section) in raw_schedule {
            schedule_map
                .entry((week, day, place))
                .or_default()
                .insert(section);
        }

        let schedule = schedule_map
            .into_iter()
            .map(|((week, day, place), time)| GraduateCourseSchedule {
                week,
                day,
                place,
                time: time.into_iter().collect(),
            })
            .collect();

        courses.push(GraduateCourse {
            course_id,
            course_name,
            class_name,
            teacher: if teacher.is_empty() {
                None
            } else {
                Some(teacher)
            },
            schedule,
        });
    }

    Ok(courses)
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
) -> Result<Vec<GraduateCourse>, crate::Error<TokenExpired>> {
    let raw_rows = raw::raw_class_table_data(yjsxt_token, termcode).await?;
    build_graduate_course_info(&raw_rows)
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
