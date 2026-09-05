use crate::{
    error::{MapParseErr, parse_err},
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
        Regex::new(r"上课时间:.*\[([0-9\-]+)周\](连续周|单周|双周)")
            .unwrap_or_else(|e| panic!("创建正则表达式失败: {:?}", e))
    });
    static TEACHER_AND_CLASSROOM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(.*)\[(.*)\]").unwrap_or_else(|e| panic!("创建正则表达式失败: {:?}", e))
    });

    let parts: Vec<&str> = cell_text.split("<br/>").filter(|s| !s.is_empty()).collect();

    // 研究生系统返回的信息可能有多余的神秘空格，要去掉
    let course_id = parts
        .first()
        .ok_or_else(|| parse_err("找不到课程编号", cell_text))?
        .replace("课程编号:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let course_name = parts
        .get(1)
        .ok_or_else(|| parse_err("找不到课程名称", cell_text))?
        .replace("课程名称:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let class_name = parts
        .get(2)
        .ok_or_else(|| parse_err("找不到班级", cell_text))?
        .replace("班级:", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    // 上课时间: [9-16周] 连续周，也可能是 单周/双周
    let class_time_str = parts
        .get(3)
        .ok_or_else(|| parse_err("找不到上课时间", cell_text))?
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let class_time = CLASS_TIME_REGEX
        .captures(&class_time_str)
        .ok_or_else(|| parse_err("解析上课时间失败", &class_time_str))?;
    let week_range = class_time
        .get(1)
        .and_then(|c| {
            c.as_str()
                .split('-')
                .map(|s| s.parse::<u8>().ok())
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| parse_err("解析上课时间失败", &class_time_str))?;
    let Some(weeks_l) = week_range.first() else {
        return Err(parse_err("解析上课时间失败", &class_time_str));
    };
    // 可能只有一个周次
    let weeks_r = week_range.get(1).unwrap_or(weeks_l);
    // 单周只有奇数周上课，双周只有偶数周上课
    let weeks: HashSet<u8> = (*weeks_l..=*weeks_r)
        .filter(|week| match class_time.get(2).map(|m| m.as_str()) {
            Some("单周") => week % 2 == 1,
            Some("双周") => week % 2 == 0,
            _ => true,
        })
        .collect();

    let teacher_and_classroom_str = parts
        .get(4)
        .ok_or_else(|| parse_err("找不到授课老师和上课地点", cell_text))?
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
        return Err(parse_err(
            "解析授课老师和上课地点失败",
            &teacher_and_classroom_str,
        ));
    };
    let [_, teacher, classroom] = teacher_and_classroom
        .try_into()
        .map_err(|_| parse_err("解析授课老师和上课地点失败", &teacher_and_classroom_str))?;

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
    Ok((res, weeks, classroom))
}

/// `json_str` 为 [super::fetch::class_table] 的返回数据
pub fn class_table(json_str: &str) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let json = serde_json::from_str::<Value>(json_str).parse_err(json_str)?;
    let raw_rows = json
        .get("rows")
        .and_then(|rows| rows.as_array())
        .ok_or_else(|| parse_err("无法解析课表行", json_str))?;

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
            .ok_or_else(|| parse_err("解析节次失败", &item.to_string()))?;
        for day in 1..=7u8 {
            let key = format!("z{day}");
            if item[&key] == Value::Null {
                continue;
            }

            let cell_text = item[&key]
                .as_str()
                .ok_or_else(|| parse_err("解析课表单元格文本失败", &item.to_string()))?;
            let (course_info, weeks, place) = parse_course_info(cell_text)?;

            if jc == "无节次" {
                extra_courses.insert(course_info, ());
            } else {
                let jc = jc.parse::<u8>().parse_err(jc)?;
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
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_class_table() -> TestResult<()> {
        // py_kbcx_ew 接口返回的是明文 JSON，未经过加密
        let courses = class_table(include_str!("test_data/py_kbcx_ew.json"))?;
        assert_eq!(courses.len(), 6);

        fn find_course<'a>(courses: &'a [Course], id: &str) -> &'a Course {
            courses
                .iter()
                .find(|c| c.course_id == id)
                .unwrap_or_else(|| panic!("测试数据中找不到课程 '{}'", id))
        }

        // 解析会把同一课程、周次、周几、地点的节次合并，这里按 (week, day, place) 汇总便于断言
        fn schedule_map(course: &Course) -> HashMap<(u8, u8, String), Vec<u8>> {
            let schedule = course.schedule.as_ref().expect("课程应该有课表");
            schedule
                .iter()
                .map(|s| {
                    let mut time = s.time.clone();
                    time.sort_unstable();
                    ((s.week, s.day, s.place.clone()), time)
                })
                .collect()
        }

        fn expand(
            map: &mut HashMap<(u8, u8, String), Vec<u8>>,
            weeks: std::ops::RangeInclusive<u8>,
            day: u8,
            place: &str,
            times: Vec<u8>,
        ) {
            for week in weeks {
                map.insert((week, day, place.to_string()), times.clone());
            }
        }

        // 示例课程1：周一、周三各占第 1-8 周
        let c1 = find_course(&courses, "1001");
        assert_eq!(c1.course_name, "示例课程1");
        assert_eq!(c1.class_name, "示例班");
        assert_eq!(c1.teacher.as_deref(), Some("教师甲"));
        let mut expected = HashMap::new();
        expand(&mut expected, 1..=8, 1, "教学楼101", vec![1, 2]);
        expand(&mut expected, 1..=8, 3, "教学楼101", vec![3, 4]);
        assert_eq!(schedule_map(c1), expected);

        // 示例课程2：周四占第 1-16 周
        let c2 = find_course(&courses, "1002");
        assert_eq!(c2.course_name, "示例课程2");
        assert_eq!(c2.teacher.as_deref(), Some("教师乙"));
        let mut expected = HashMap::new();
        expand(&mut expected, 1..=16, 4, "教学楼102", vec![2, 3, 4]);
        assert_eq!(schedule_map(c2), expected);

        // 示例课程3：周四、周五各占第 9-16 周
        let c3 = find_course(&courses, "1003");
        assert_eq!(c3.course_name, "示例课程3");
        assert_eq!(c3.teacher.as_deref(), Some("教师丙"));
        let mut expected = HashMap::new();
        expand(&mut expected, 9..=16, 5, "教学楼103", vec![3, 4]);
        expand(&mut expected, 9..=16, 4, "教学楼103", vec![9, 10]);
        assert_eq!(schedule_map(c3), expected);

        // 示例课程4：周五占第 1-11 周
        let c4 = find_course(&courses, "1004");
        assert_eq!(c4.course_name, "示例课程4");
        assert_eq!(c4.teacher.as_deref(), Some("教师丁"));
        let mut expected = HashMap::new();
        expand(&mut expected, 1..=11, 5, "教学楼104", vec![5, 6, 7]);
        assert_eq!(schedule_map(c4), expected);

        // 示例课程5：周一、周四各占第 1-8 周
        let c5 = find_course(&courses, "1005");
        assert_eq!(c5.course_name, "示例课程5");
        assert_eq!(c5.teacher.as_deref(), Some("教师戊"));
        let mut expected = HashMap::new();
        expand(&mut expected, 1..=8, 4, "教学楼105", vec![7, 8]);
        expand(&mut expected, 1..=8, 1, "教学楼105", vec![9, 10]);
        assert_eq!(schedule_map(c5), expected);

        // 示例课程6：单周课程，周二占第 1-15 周中的奇数周
        let c6 = find_course(&courses, "1006");
        assert_eq!(c6.course_name, "示例课程6");
        assert_eq!(c6.teacher.as_deref(), Some("教师己"));
        let mut expected = HashMap::new();
        for week in (1..=15).step_by(2) {
            expected.insert((week, 2, "教学楼106".to_string()), vec![1, 2]);
        }
        assert_eq!(schedule_map(c6), expected);

        Ok(())
    }
}
