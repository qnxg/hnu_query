use crate::{
    error::{MapParseErr, parse_err, parse_err_with_reason},
    yjsxt::{
        class_table::{Course, CourseSchedule},
        error::TokenExpired,
    },
};
use regex::Regex;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

#[derive(Hash, Eq, PartialEq)]
struct ParsedCourse {
    course_name: String,
    course_id: String,
    class_name: String,
    teacher: Option<String>,
}

fn parse_course_info(
    cell_text: &str,
) -> Result<(ParsedCourse, HashSet<u8>, String), crate::Error<TokenExpired>> {
    static CLASS_TIME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"上课时间:.*\[([0-9\-]+)周\].*连续周")
            .unwrap_or_else(|e| panic!("创建正则表达式失败: {:?}", e))
    });
    static TEACHER_AND_CLASSROOM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(.*)\[(.*)\]").unwrap_or_else(|e| panic!("创建正则表达式失败: {:?}", e))
    });

    let parts: Vec<&str> = cell_text.split("<br/>").filter(|s| !s.is_empty()).collect();

    // 研究生系统返回的信息可能有多余的神秘空格，要去掉
    let course_id = parts
        .first()
        .ok_or_else(|| parse_err(cell_text))?
        .replace("课程编号:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let course_name = parts
        .get(1)
        .ok_or_else(|| parse_err(cell_text))?
        .replace("课程名称:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let class_name = parts
        .get(2)
        .ok_or_else(|| parse_err(cell_text))?
        .replace("班级:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    // 上课时间: [9-16周] 连续周
    let class_time_str = parts
        .get(3)
        .ok_or_else(|| parse_err(cell_text))?
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let class_time = CLASS_TIME_REGEX
        .captures(&class_time_str)
        .and_then(|c| c.get(1))
        .and_then(|c| {
            c.as_str()
                .split('-')
                .map(|s| s.parse::<u8>().ok())
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| parse_err_with_reason(&class_time_str, "解析上课时间失败"))?;
    let Some(weeks_l) = class_time.first() else {
        return Err(parse_err_with_reason(&class_time_str, "解析上课时间失败"));
    };
    // 可能只有一个周次
    let weeks_r = class_time.get(1).unwrap_or(weeks_l);

    let teacher_and_classroom_str = parts
        .get(4)
        .ok_or_else(|| parse_err(cell_text))?
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let Some(teacher_and_classroom) = TEACHER_AND_CLASSROOM_REGEX
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

/// `json_str` 为 [super::fetch::class_table] 的返回数据
pub fn class_table(json_str: &str) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let json_str = crate::yjsxt::parse::decrypt_response(json_str)?;
    let json = serde_json::from_str::<Value>(&json_str).parse_err(&json_str)?;
    let raw_rows = json
        .get("rows")
        .and_then(|rows| rows.as_array())
        .ok_or_else(|| parse_err(&json_str))?;

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
            .ok_or_else(|| parse_err_with_reason(&item.to_string(), "解析节次失败"))?;
        for day in 1..=7u8 {
            let key = format!("z{day}");
            if item[&key] == Value::Null {
                continue;
            }

            let cell_text = item[&key]
                .as_str()
                .ok_or_else(|| parse_err(&item.to_string()))?;
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
