use super::{Course, CourseSchedule, ExtraCourse};
use crate::{
    error::{MapParseErr, parse_err, parse_err_with_reason},
    hdjw::error::TokenExpired,
};
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// 教务 `教学运行 > 我的课表 > 有课表课程` 返回数据单项
/// 还有其他一些具体学时信息的字段，懒得搞了
#[derive(Deserialize, Debug)]
#[expect(unused)]
struct RawCourseInfo {
    /// 课程代码
    kch: String,
    /// 课程名称
    kc_mc: String,
    /// 教师名称
    jg0101mc: Option<String>,
    /// 教师工号（暂时不用）
    jsgh: Option<String>,
    kt_mc: String, // 上课班级
    /// 课堂容量（暂时不用）
    pkrs: u16,
    /// 上课人数
    xkrs: u16,
    /// 课程性质（通识必修/专业核心等）
    kcxz: String,
    /// 课程类别（必修/选修等）
    kclb: String,
    /// 通知单编号（暂时不用）
    jx0404id: String,
    /// 分组名称，这里当作课程的备注信息
    fzmc: Option<String>,
    /// 上课时间
    sktime: String,
    /// 上课地点
    skddmc: String,
    /// 上课校区
    skxqmc: String,
    /// 开课院系（暂时不用）
    kkyx: String,
    /// 周学时（暂时不用）
    zhouxs: String,
    /// 学分
    xf: f32,
    /// 总学时（暂时不用）
    zxs: u16,
    /// 考核方式（暂时不用）
    khfs: String,
}

/// 教务 `教学运行 > 我的课表 > 无课表课程` 返回数据单项
#[derive(Deserialize, Debug)]
struct RawExtraCourseInfo {
    /// 课程代码
    kch: String,
    /// 课程名称
    kc_mc: String,
    /// 教师名称
    jg0101mc: String,
    /// 分组名称
    fzmc: Option<String>,
    /// 课程性质（通识必修/专业核心等）
    kcxz: String,
    /// 上课班级
    kt_mc: String,
    /// 上课人数
    xkrs: u16,
    /// 上课校区
    skxqmc: String,
    /// 学分
    xf: f32,
}

fn day_to_u8(c: char) -> Result<u8, crate::Error<TokenExpired>> {
    match c {
        '一' => Ok(1),
        '二' => Ok(2),
        '三' => Ok(3),
        '四' => Ok(4),
        '五' => Ok(5),
        '六' => Ok(6),
        '日' | '七' => Ok(7),
        _ => Err(parse_err_with_reason(
            &format!("未知的星期字符: {c}"),
            "上课时间: day",
        )),
    }
}

fn extract_time_list(s: &str, context: &str) -> Result<HashSet<u8>, crate::Error<TokenExpired>> {
    // 节次信息首先由 、分割，分割出来的每个部分即可能是一个单个数字，有可能是一个区间范围（由 - 连接）
    let mut time_set = HashSet::new();
    for time_range_str in s.split('、') {
        let parts: Vec<_> = time_range_str.split('-').collect();
        let time_l = parts
            .first()
            .and_then(|v| v.parse::<u8>().ok())
            .ok_or(parse_err_with_reason(context, "上课时间: time"))?;
        let time_r = match parts.get(1) {
            Some(v) => v
                .parse::<u8>()
                .parse_err_with_reason(context, "上课时间: time")?,
            None => time_l,
        };
        time_set.extend(time_l..=time_r);
    }
    Ok(time_set)
}

fn extract_week_list(s: &str, context: &str) -> Result<HashSet<u8>, crate::Error<TokenExpired>> {
    // 周次信息首先由 , 分割，分割出来的每个部分即可能是一个单个数字，有可能是一个区间范围（由 - 连接）
    let mut week_set = HashSet::new();
    for week_range_str in s.split(',') {
        let parts: Vec<_> = week_range_str.split('-').collect();
        let week_l = parts
            .first()
            .and_then(|v| v.parse::<u8>().ok())
            .ok_or(parse_err_with_reason(context, "上课时间: week"))?;
        let week_r = match parts.get(1) {
            Some(v) => v
                .parse::<u8>()
                .parse_err_with_reason(context, "上课时间: week")?,
            None => week_l,
        };
        week_set.extend(week_l..=week_r);
    }
    Ok(week_set)
}

static SKTIME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"周(.)第(.*)节.*\{第(.*)周\}")
        .unwrap_or_else(|e| panic!("创建正则表达式失败: {:?}", e))
});

/// 解析上课时间地点
fn course_schedule(raw: &RawCourseInfo) -> Result<Vec<CourseSchedule>, crate::Error<TokenExpired>> {
    let places: Vec<_> = raw.skddmc.split(';').collect();
    let detail_times = raw.sktime.split(';');
    // 第几周+周几+地点作为 key，节次作为 value，进行去重
    let mut schedule: HashMap<(u8, u8, String), HashSet<u8>> = HashMap::new();
    for (i, time) in detail_times.into_iter().enumerate() {
        let caps = SKTIME_RE
            .captures(time)
            .ok_or(parse_err_with_reason(&raw.sktime, "上课时间: day"))?;
        let day = day_to_u8(caps.get(1).and_then(|v| v.as_str().chars().next()).ok_or(
            parse_err_with_reason(&raw.sktime, "上课时间: day: 没有匹配到星期字符"),
        )?)?;
        let time_list = extract_time_list(
            caps.get(2)
                .ok_or(parse_err_with_reason(&raw.sktime, "上课时间: time"))?
                .as_str(),
            &raw.sktime,
        )?;
        let week_list = extract_week_list(
            caps.get(3)
                .ok_or(parse_err_with_reason(&raw.sktime, "上课时间: week"))?
                .as_str(),
            &raw.sktime,
        )?;
        let place = places
            .get(i)
            .ok_or(parse_err_with_reason(&raw.skddmc, "上课地点"))?;
        week_list.iter().for_each(|&week| {
            schedule
                .entry((week, day, place.to_string()))
                .or_default()
                .extend(time_list.iter());
        });
    }
    Ok(schedule
        .into_iter()
        .map(|((week, day, place), time)| CourseSchedule {
            week,
            day,
            place,
            time: time.into_iter().collect(),
        })
        .collect())
}

/// `json_str` 为 [`super::fetch::get_xskb_list`] 返回的数据
pub fn class_table(json_str: &str) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let json = crate::hdjw::parse::hdjw_response(json_str)?;
    let raw_data = match json.get("count").and_then(|c| c.as_u64()) {
        None => return Err(parse_err(json_str)),
        Some(0) => return Ok(vec![]), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => serde_json::from_value::<Vec<RawCourseInfo>>(json["data"].clone())
            .parse_err(json_str)?,
    };
    let mut courses = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let schedule = course_schedule(&item)?;
        courses.push(Course {
            course_name: item.kc_mc,
            course_id: item.kch,
            course_type: item.kcxz,
            class_name: item.kt_mc,
            area: item.skxqmc,
            // 教务系统可能会返回空格开头或结尾
            teacher: item.jg0101mc.map(|s| s.trim().to_string()),
            credit: item.xf,
            extra: item.fzmc,
            people: item.xkrs,
            schedule,
        });
    }
    Ok(courses)
}

/// # Parameters
///
/// - `raw_data`: 由 [`super::raw::get_xskb_list_extra`] 返回的数据
pub fn class_table_extra(json_str: &str) -> Result<Vec<ExtraCourse>, crate::Error<TokenExpired>> {
    let json = crate::hdjw::parse::hdjw_response(json_str)?;
    let raw_data = match json.get("count").and_then(|c| c.as_u64()) {
        None => return Err(parse_err(json_str)),
        Some(0) => return Ok(vec![]), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => serde_json::from_value::<Vec<RawExtraCourseInfo>>(json["data"].clone())
            .parse_err(json_str)?,
    };
    let mut courses = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        courses.push(ExtraCourse {
            course_name: item.kc_mc,
            course_id: item.kch,
            course_type: item.kcxz,
            class_name: item.kt_mc,
            area: item.skxqmc,
            teacher: item.jg0101mc,
            credit: item.xf,
            extra: item.fzmc,
            people: item.xkrs,
        });
    }
    Ok(courses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_day_to_u8() -> TestResult<()> {
        assert_eq!(day_to_u8('一')?, 1);
        assert_eq!(day_to_u8('五')?, 5);
        assert_eq!(day_to_u8('日')?, 7);
        assert_eq!(day_to_u8('七')?, 7);

        assert!(day_to_u8('A').is_err());

        Ok(())
    }

    #[test]
    fn test_time_list() -> TestResult<()> {
        assert_eq!(extract_time_list("3", "test")?, HashSet::from([3]));
        assert_eq!(
            extract_time_list("3、4-5", "test")?,
            HashSet::from([3, 4, 5])
        );

        Ok(())
    }

    #[test]
    fn test_week_list() -> TestResult<()> {
        assert_eq!(extract_week_list("1", "test")?, HashSet::from([1]));
        assert_eq!(extract_week_list("1-16", "test")?, (1..=16).collect());
        assert_eq!(
            extract_week_list("1-2,5-6", "test")?,
            HashSet::from([1, 2, 5, 6])
        );

        Ok(())
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn test_class_table() -> TestResult<()> {
        let courses = class_table(include_str!("test_data/xskb_list.json"))?;

        fn sorted(mut v: Vec<u8>) -> Vec<u8> {
            v.sort();
            v
        }

        fn find_course<'a>(courses: &'a [Course], name: &str) -> &'a Course {
            courses
                .iter()
                .find(|c| c.course_name == name)
                .unwrap_or_else(|| panic!("测试数据中找不到课程 '{}'", name))
        }

        assert_eq!(courses.len(), 8);

        // 高等数学AⅡ -- 最复杂：6 个时间段、周间隔、单周、日/七 映射、去重
        let test_course_1 = find_course(&courses, "高等数学AⅡ");
        assert_eq!(test_course_1.course_id, "ZJ001SX24AⅡ");
        assert_eq!(test_course_1.credit, 5.0);
        assert_eq!(test_course_1.course_type, "专业基础");
        assert_eq!(test_course_1.class_name, "大数据管理[2501-2503]班");
        assert_eq!(test_course_1.area, "南校区");
        assert_eq!(test_course_1.teacher, Some("马教师".to_string()));
        assert_eq!(test_course_1.extra, None);
        assert_eq!(test_course_1.people, 104);
        assert_eq!(test_course_1.schedule.len(), 47);

        // 周一 第1、2节{第1-9,11-15周} → 14 entries
        let day1: Vec<_> = test_course_1
            .schedule
            .iter()
            .filter(|s| s.day == 1)
            .collect();
        assert_eq!(day1.len(), 14);
        let day1_weeks: HashSet<u8> = day1.iter().map(|s| s.week).collect();
        assert_eq!(day1_weeks, (1..=9).chain(11..=15).collect());
        for s in &day1 {
            assert_eq!(s.place, "综204");
            assert_eq!(sorted(s.time.clone()), vec![1, 2]);
        }

        // 周三 第3、4节{第1-15周} → 15 entries
        let day3: Vec<_> = test_course_1
            .schedule
            .iter()
            .filter(|s| s.day == 3)
            .collect();
        assert_eq!(day3.len(), 15);
        let day3_weeks: HashSet<u8> = day3.iter().map(|s| s.week).collect();
        assert_eq!(day3_weeks, (1..=15).collect());
        for s in &day3 {
            assert_eq!(s.place, "综204");
            assert_eq!(sorted(s.time.clone()), vec![3, 4]);
        }

        // 周五 第1、2节{第1-3,5-16周} → 15 entries
        let day5: Vec<_> = test_course_1
            .schedule
            .iter()
            .filter(|s| s.day == 5)
            .collect();
        assert_eq!(day5.len(), 15);
        let day5_weeks: HashSet<u8> = day5.iter().map(|s| s.week).collect();
        assert_eq!(day5_weeks, (1..=3).chain(5..=16).collect());
        for s in &day5 {
            assert_eq!(s.place, "综204");
            assert_eq!(sorted(s.time.clone()), vec![1, 2]);
        }

        // 周六 第14周 + 周六 第10周 → 2 entries
        let day6: Vec<_> = test_course_1
            .schedule
            .iter()
            .filter(|s| s.day == 6)
            .collect();
        assert_eq!(day6.len(), 2);
        let day6_weeks: HashSet<u8> = day6.iter().map(|s| s.week).collect();
        assert_eq!(day6_weeks, HashSet::from([10, 14]));
        for s in &day6 {
            assert_eq!(s.place, "综204");
            assert_eq!(sorted(s.time.clone()), vec![1, 2]);
        }

        // 周日 第15周 → 1 entry
        let day7: Vec<_> = test_course_1
            .schedule
            .iter()
            .filter(|s| s.day == 7)
            .collect();
        assert_eq!(day7.len(), 1);
        assert_eq!(day7[0].week, 15);
        assert_eq!(day7[0].place, "综204");
        assert_eq!(sorted(day7[0].time.clone()), vec![1, 2]);

        // 高级程序设计 -- 简单：单天单时间段
        let test_course_2 = find_course(&courses, "高级程序设计");
        assert_eq!(test_course_2.course_id, "ZH016GS24");
        assert_eq!(test_course_2.credit, 2.0);
        assert_eq!(test_course_2.course_type, "专业核心");
        assert_eq!(test_course_2.class_name, "大数据管理2501班");
        assert_eq!(test_course_2.area, "南校区");
        assert_eq!(test_course_2.teacher, Some("何教师,傅教,李教".to_string()));
        assert_eq!(test_course_2.extra, None);
        assert_eq!(test_course_2.people, 35);
        assert_eq!(test_course_2.schedule.len(), 12);
        let prog_weeks: HashSet<u8> = test_course_2.schedule.iter().map(|s| s.week).collect();
        assert_eq!(prog_weeks, (1..=12).collect());
        for s in &test_course_2.schedule {
            assert_eq!(s.day, 4);
            assert_eq!(s.place, "研B205");
            assert_eq!(sorted(s.time.clone()), vec![7, 8]);
        }

        // 桥梁历史、文化与技术 -- 时间区间：第9-11节
        let test_course_3 = find_course(&courses, "桥梁历史、文化与技术");
        assert_eq!(test_course_3.course_id, "TK005TM24");
        assert_eq!(test_course_3.credit, 2.0);
        assert_eq!(test_course_3.course_type, "通识选修");
        assert_eq!(test_course_3.class_name, "教学班");
        assert_eq!(test_course_3.area, "南校区");
        assert_eq!(
            test_course_3.teacher,
            Some("刘教师,王教师,华教师".to_string())
        );
        assert_eq!(test_course_3.extra, None);
        assert_eq!(test_course_3.people, 89);
        assert_eq!(test_course_3.schedule.len(), 11);
        let bridge_weeks: HashSet<u8> = test_course_3.schedule.iter().map(|s| s.week).collect();
        assert_eq!(bridge_weeks, (3..=13).collect());
        for s in &test_course_3.schedule {
            assert_eq!(s.day, 3);
            assert_eq!(s.place, "复308");
            assert_eq!(sorted(s.time.clone()), vec![9, 10, 11]);
        }

        Ok(())
    }

    #[test]
    fn test_class_table_extra() -> TestResult<()> {
        let courses = class_table_extra(include_str!("test_data/xskb_list_extra.json"))?;

        assert_eq!(courses.len(), 1);

        let course = &courses[0];
        assert_eq!(course.course_id, "TB001XG24");
        assert_eq!(course.course_name, "大学生心理健康教育");
        assert_eq!(course.course_type, "通识必修");
        assert_eq!(
            course.class_name,
            "文学[2501-2502]班,大数据管理[2501-2503]班,工管[2501-2503]班,会计[2501-2503]班"
        );
        assert_eq!(course.area, "南校区");
        assert_eq!(course.teacher, "张教师");
        assert_eq!(course.credit, 1.0);
        assert_eq!(course.extra, Some("全自主在线学习".to_string()));
        assert_eq!(course.people, 338);

        Ok(())
    }
}
