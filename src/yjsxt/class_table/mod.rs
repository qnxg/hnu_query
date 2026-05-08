mod raw;

use std::collections::HashMap;
use std::sync::LazyLock;

use serde_json::Value;

use crate::{
    error::{MapParseErr, parse_err},
    hdjw::class_table::{Course, CourseSchedule},
    yjsxt::{error::TokenExpired, login::YjsxtToken},
};

/// 研究生课程原始信息
#[derive(Debug)]
pub struct GraduateCourseInfo {
    pub course_id: String,
    pub course_name: String,
    pub teacher: String,
    pub class_name: String,
    pub place: String,
    pub area: String,
    pub day: u8,
    pub sections: String,
    pub weeks: String,
    pub start_time: String,
    pub end_time: String,
    pub course_type: String,
    pub credit: f32,
    pub extra: Option<String>,
}

static START_TIMES: LazyLock<HashMap<u8, &str>> = LazyLock::new(|| {
    HashMap::from([
        (1, "8:00"),
        (2, "8:55"),
        (3, "10:00"),
        (4, "10:55"),
        (5, "14:30"),
        (6, "15:15"),
        (7, "16:10"),
        (8, "16:55"),
        (9, "19:00"),
        (10, "19:55"),
        (11, "20:50"),
        (12, "21:35"),
    ])
});

static END_TIMES: LazyLock<HashMap<u8, &str>> = LazyLock::new(|| {
    HashMap::from([
        (1, "8:45"),
        (2, "9:40"),
        (3, "10:45"),
        (4, "11:40"),
        (5, "15:15"),
        (6, "16:00"),
        (7, "16:55"),
        (8, "17:40"),
        (9, "19:45"),
        (10, "20:40"),
        (11, "21:35"),
        (12, "22:20"),
    ])
});

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

fn build_graduate_course_info(
    rows: &[Value],
) -> Result<Vec<GraduateCourseInfo>, crate::Error<TokenExpired>> {
    let mut courses: Vec<GraduateCourseInfo> = Vec::new();

    for item in rows {
        if item["mc"] == Value::String("无节次".to_string()) {
            continue;
        }
        let jc = item["mc"]
            .as_str()
            .ok_or(parse_err(&serde_json::to_string(&item).unwrap_or_default()))?
            .parse::<u8>()
            .parse_err_with_reason("", "解析节次失败")?;
        let section_id = format!("{:0>2}", jc);

        for day in 1..=7u8 {
            let key = format!("z{day}");
            if item[&key] == Value::Null {
                continue;
            }
            let cell_text = item[&key]
                .as_str()
                .ok_or(parse_err(&item.to_string()))?;
            let course_info = parse_course_info(cell_text).ok_or(parse_err(cell_text))?;

            // 尝试与已有课程合并（连续节次）
            let mut merged = false;
            for existing in courses.iter_mut() {
                if existing.course_name == course_info.course_name
                    && existing.weeks == course_info.class_time
                    && existing.teacher == course_info.teacher
                    && existing.day == day
                {
                    let existing_sections: Vec<u8> = existing
                        .sections
                        .split(',')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if existing_sections.contains(&(jc - 1)) {
                        existing.sections = format!("{},{}", existing.sections, section_id);
                        existing.end_time = END_TIMES[&jc].to_string();
                        merged = true;
                        break;
                    }
                }
            }

            if !merged {
                let mut course_id = course_info.course_id.clone();
                let count = courses
                    .iter()
                    .filter(|c| c.course_id.starts_with(&course_info.course_id))
                    .count();
                if count > 0 {
                    course_id = format!("{}_{}", course_id, count + 1);
                }
                courses.push(GraduateCourseInfo {
                    course_id,
                    course_name: course_info.course_name,
                    teacher: course_info.teacher,
                    class_name: course_info.class_name,
                    place: course_info.classroom,
                    area: String::new(),
                    day,
                    sections: section_id.clone(),
                    weeks: course_info.class_time,
                    start_time: START_TIMES[&jc].to_string(),
                    end_time: END_TIMES[&jc].to_string(),
                    course_type: String::new(),
                    credit: 0.0,
                    extra: None,
                });
            }
        }
    }

    Ok(courses)
}

fn parse_graduate_course_info(
    raw_data: Vec<GraduateCourseInfo>,
) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let mut courses = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let time_list: Vec<u8> = item
            .sections
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        let week_list: Vec<u8> = item
            .weeks
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        let schedule: Vec<CourseSchedule> = week_list
            .into_iter()
            .map(|week| CourseSchedule {
                week,
                day: item.day,
                place: item.place.clone(),
                time: time_list.clone(),
            })
            .collect();
        let course = Course {
            course_name: item.course_name,
            course_id: item.course_id,
            course_type: item.course_type,
            class_name: item.class_name,
            area: item.area,
            teacher: if item.teacher.is_empty() {
                None
            } else {
                Some(item.teacher)
            },
            credit: item.credit,
            extra: item.extra,
            people: 0,
            schedule,
        };
        courses.push(course);
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
) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let raw_rows = raw::raw_class_table_data(yjsxt_token, termcode).await?;
    let graduate_courses = build_graduate_course_info(&raw_rows)?;
    parse_graduate_course_info(graduate_courses)
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
