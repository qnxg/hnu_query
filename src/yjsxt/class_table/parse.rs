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

/// 课程信息、上课周次集合、上课地点
type ParsedCourseBlock = (ParsedCourse, HashSet<u8>, String);

/// 解析课表单元格文本
///
/// 一个单元格可能包含多个课程块，块之间以空行分隔，每个块以“课程编号:”开头
fn parse_course_info(
    cell_text: &str,
) -> Result<Vec<ParsedCourseBlock>, crate::Error<TokenExpired>> {
    let parts: Vec<&str> = cell_text
        .split("<br/>")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    // 按“课程编号:”把单元格切分为多个课程块。
    // 注意：第一个“课程编号:”之前不允许出现非空行，否则说明有内容被漏掉，必须报错
    let blocks: Vec<&[&str]> = parts
        .chunk_by(|_, next| !next.starts_with("课程编号:"))
        .collect();
    let Some(first) = blocks.first() else {
        return Err(parse_err("找不到课程编号", cell_text));
    };
    if !first[0].starts_with("课程编号:") {
        return Err(parse_err("课程块之前存在无法识别的内容", cell_text));
    }

    blocks
        .into_iter()
        .map(|block| parse_course_block(block, cell_text))
        .collect()
}

/// 去掉课程块一行的前缀和多余的空白字符
///
/// 前缀不匹配说明行的内容或顺序与预期不符，属于未知格式，报错
fn strip_line_prefix(
    line: &str,
    prefix: &str,
    cell_text: &str,
) -> Result<String, crate::Error<TokenExpired>> {
    let s = line
        .strip_prefix(prefix)
        .ok_or_else(|| parse_err("课程块格式异常", cell_text))?;
    Ok(s.chars().filter(|c| !c.is_whitespace()).collect())
}

/// 解析单个课程块，应为 课程编号/课程名称/班级/上课时间/老师和地点 共 5 行
fn parse_course_block(
    block: &[&str],
    cell_text: &str,
) -> Result<ParsedCourseBlock, crate::Error<TokenExpired>> {
    // 注意必须全行锚定匹配：若一行内出现多个周次区间，
    // 非锚定的贪婪匹配会静默只取最后一个，丢弃前面的区间
    static CLASS_TIME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^上课时间:\[([0-9\-]+)周\](.*)$")
            .unwrap_or_else(|e| panic!("创建正则表达式失败: {:?}", e))
    });
    static TEACHER_AND_CLASSROOM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(.*)\[(.*)\]").unwrap_or_else(|e| panic!("创建正则表达式失败: {:?}", e))
    });

    // 块内出现多余或缺失的行说明格式未知，直接报错
    if block.len() != 5 {
        return Err(parse_err("课程块格式异常", cell_text));
    }

    // 研究生系统返回的信息可能有多余的神秘空格，要去掉
    // 每行都必须以预期的前缀开头，否则说明格式未知，直接报错
    let course_id = strip_line_prefix(block[0], "课程编号:", cell_text)?;
    let course_name = strip_line_prefix(block[1], "课程名称:", cell_text)?;
    let class_name = strip_line_prefix(block[2], "班级:", cell_text)?;

    // 上课时间: [9-16周] 连续周，也可能是 单周/双周
    let class_time_str = block[3]
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
    // 周次区间最多两段（如 9-16），出现更多段说明格式未知，必须报错而不是静默丢弃
    if week_range.len() > 2 {
        return Err(parse_err("解析上课时间失败", &class_time_str));
    }
    // 可能只有一个周次
    let weeks_r = week_range.get(1).unwrap_or(weeks_l);
    // 单周只有奇数周上课，双周只有偶数周上课
    // 未知的周次类型报错
    let parity = match class_time.get(2).map(|m| m.as_str()) {
        Some("连续周") => None,
        Some("单周") => Some(1),
        Some("双周") => Some(0),
        _ => return Err(parse_err("未知的周次类型", &class_time_str)),
    };
    let weeks: HashSet<u8> = (*weeks_l..=*weeks_r)
        .filter(|week| parity.is_none_or(|p| week % 2 == p))
        .collect();

    let teacher_and_classroom_str = block[4]
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

            for (course_info, weeks, place) in parse_course_info(cell_text)? {
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
    fn test_unknown_week_type_errors() {
        // 未知的周次类型应报错
        let cell = "<br/>课程编号:1007<br/>课程名称:示例课程7<br/>班级:示例班<br/>上课时间:[1-15周]三周<br/>教师庚[教学楼107]";
        assert!(parse_course_info(cell).is_err());
    }

    #[test]
    fn test_unconsumed_content_errors() {
        // 一行内出现多个周次区间属于未知格式，必须报错而不是只解析其中一个
        let cell = "<br/>课程编号:1007<br/>课程名称:示例课程7<br/>班级:示例班<br/>上课时间:[1-4周]连续周[5-8周]连续周<br/>教师庚[教学楼107]";
        assert!(parse_course_info(cell).is_err());

        // 课程块之前出现无法识别的内容必须报错
        let cell = "<br/>备注:这是一行未知内容<br/>课程编号:1007<br/>课程名称:示例课程7<br/>班级:示例班<br/>上课时间:[1-15周]连续周<br/>教师庚[教学楼107]";
        assert!(parse_course_info(cell).is_err());

        // 行的前缀与预期不符必须报错
        let cell = "<br/>课程编号:1007<br/>课程名称:示例课程7<br/>教学班:示例班<br/>上课时间:[1-15周]连续周<br/>教师庚[教学楼107]";
        assert!(parse_course_info(cell).is_err());

        // 周次区间超过两段属于未知格式，必须报错
        let cell = "<br/>课程编号:1007<br/>课程名称:示例课程7<br/>班级:示例班<br/>上课时间:[1-2-3周]连续周<br/>教师庚[教学楼107]";
        assert!(parse_course_info(cell).is_err());
    }

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

    #[test]
    fn test_class_table() -> TestResult<()> {
        // py_kbcx_ew 接口返回的是明文 JSON，未经过加密
        let courses = class_table(include_str!("test_data/py_kbcx_ew.json"))?;
        assert_eq!(courses.len(), 5);

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

        Ok(())
    }

    #[test]
    fn test_class_table_multi_teacher_phases() -> TestResult<()> {
        // 一门课分阶段由两个老师授课时，同一单元格内会包含两个课程块，
        // 两个阶段都要解析出来（此处按老师拆分为两门课程记录）
        let courses = class_table(include_str!("test_data/py_kbcx_ew_multi_phase.json"))?;
        assert_eq!(courses.len(), 12);

        let mut phases = courses
            .iter()
            .filter(|c| c.course_id == "1018")
            .collect::<Vec<_>>();
        assert_eq!(phases.len(), 2);
        // 按起始周次排序，区分两个阶段
        phases.sort_by_key(|c| {
            c.schedule
                .as_ref()
                .and_then(|s| s.iter().map(|s| s.week).min())
        });

        // 第一阶段：1-4 周，教师辛
        let mut expected = HashMap::new();
        expand(&mut expected, 1..=4, 5, "文科楼101", vec![9, 10]);
        assert_eq!(phases[0].teacher.as_deref(), Some("教师辛"));
        assert_eq!(schedule_map(phases[0]), expected);

        // 第二阶段：5-8 周，教师未
        let mut expected = HashMap::new();
        expand(&mut expected, 5..=8, 5, "文科楼101", vec![9, 10]);
        assert_eq!(phases[1].teacher.as_deref(), Some("教师未"));
        assert_eq!(schedule_map(phases[1]), expected);

        Ok(())
    }

    #[test]
    fn test_class_table_odd_even_weeks() -> TestResult<()> {
        // 单双周课程：单周只在奇数周上课，双周只在偶数周上课
        let courses = class_table(include_str!("test_data/py_kbcx_ew_dsz.json"))?;
        assert_eq!(courses.len(), 2);

        // 示例课程6：单周课程，周二占第 1-15 周中的奇数周
        let c6 = find_course(&courses, "1006");
        assert_eq!(c6.course_name, "示例课程6");
        assert_eq!(c6.teacher.as_deref(), Some("教师己"));
        let mut expected = HashMap::new();
        for week in (1..=15).step_by(2) {
            expected.insert((week, 2, "教学楼106".to_string()), vec![1, 2]);
        }
        assert_eq!(schedule_map(c6), expected);

        // 示例课程7：双周课程，周三占第 2-16 周中的偶数周
        let c7 = find_course(&courses, "1007");
        assert_eq!(c7.course_name, "示例课程7");
        assert_eq!(c7.teacher.as_deref(), Some("教师庚"));
        let mut expected = HashMap::new();
        for week in (2..=16).step_by(2) {
            expected.insert((week, 3, "教学楼107".to_string()), vec![1, 2]);
        }
        assert_eq!(schedule_map(c7), expected);

        Ok(())
    }
}
