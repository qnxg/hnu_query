use super::{Course, CourseSchedule, ExtraCourse};
use crate::{
    error::{MapParseErr, parse_err},
    hdjw::error::TokenExpired,
};
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// 教务 `教学运行 > 我的课表 > 列表模式` 返回数据单项
/// 无课表课程不含 `sktime`/`skddmc` 字段
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
    fz_mc: Option<String>,
    /// 上课时间
    sktime: Option<String>,
    /// 上课地点
    skddmc: Option<String>,
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

fn day_to_u8(c: char) -> Result<u8, crate::Error<TokenExpired>> {
    match c {
        '一' => Ok(1),
        '二' => Ok(2),
        '三' => Ok(3),
        '四' => Ok(4),
        '五' => Ok(5),
        '六' => Ok(6),
        '日' | '七' => Ok(7),
        _ => Err(parse_err("未知的星期字符", &format!("{c}"))),
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
            .ok_or(parse_err("找不到上课节次", context))?;
        let time_r = match parts.get(1) {
            Some(v) => v.parse::<u8>().parse_err(context)?,
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
            .ok_or(parse_err("找不到上课周次", context))?;
        let week_r = match parts.get(1) {
            Some(v) => v.parse::<u8>().parse_err(context)?,
            None => week_l,
        };
        week_set.extend(week_l..=week_r);
    }
    Ok(week_set)
}

/// 解析上课时间地点
/// 调用方需保证 `sktime` 存在（无课表课程已过滤），`skddmc` 与之成对出现
fn course_schedule(raw: &RawCourseInfo) -> Result<Vec<CourseSchedule>, crate::Error<TokenExpired>> {
    let sktime = raw.sktime.as_deref().expect("sktime 应存在");
    let skddmc = raw.skddmc.as_deref().expect("skddmc 应存在");
    // sktime 格式：`星期五3、4节{ 7-8周}(全周)`（周次可能带前导空格）
    static SKTIME_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"星期(.)(.*)节.*\{\s*(.*)周\}").expect("创建正则表达式失败"));
    let places: Vec<_> = skddmc.split(';').collect();
    let detail_times = sktime.split(';');
    // 第几周+周几+地点作为 key，节次作为 value，进行去重
    let mut schedule: HashMap<(u8, u8, String), HashSet<u8>> = HashMap::new();
    for (i, time) in detail_times.into_iter().enumerate() {
        let caps = SKTIME_RE
            .captures(time)
            .ok_or(parse_err("无法解析上课时间", sktime))?;
        let day = day_to_u8(
            caps.get(1)
                .and_then(|v| v.as_str().chars().next())
                .ok_or(parse_err("找不到上课星期", sktime))?,
        )?;
        let time_list = extract_time_list(
            caps.get(2)
                .ok_or(parse_err("找不到上课节次", sktime))?
                .as_str(),
            sktime,
        )?;
        let week_list = extract_week_list(
            caps.get(3)
                .ok_or(parse_err("找不到上课周次", sktime))?
                .as_str(),
            sktime,
        )?;
        let place = places.get(i).ok_or(parse_err("找不到上课地点", skddmc))?;
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

/// `json_str` 为 [`super::fetch::class_table`] 返回的数据
pub fn class_table(json_str: &str) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let json = crate::hdjw::parse::hdjw_response(json_str)?;
    let raw_data = match json.get("count").and_then(|c| c.as_u64()) {
        None => return Err(parse_err("无法解析课程表数据", json_str)),
        Some(0) => return Ok(vec![]), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => serde_json::from_value::<Vec<RawCourseInfo>>(json["data"].clone())
            .parse_err(json_str)?,
    };
    let mut courses = Vec::with_capacity(raw_data.len());
    // 无课表课程不含 sktime 字段
    for item in raw_data.into_iter().filter(|item| item.sktime.is_some()) {
        let schedule = course_schedule(&item)?;
        courses.push(Course {
            // 教务系统可能会返回空格或制表符开头/结尾的课程名
            course_name: item.kc_mc.trim().to_string(),
            course_id: item.kch,
            course_type: item.kcxz,
            class_name: item.kt_mc,
            area: item.skxqmc,
            // 教务系统可能会返回空格开头或结尾
            teacher: item.jg0101mc.map(|s| s.trim().to_string()),
            credit: item.xf,
            extra: item.fz_mc,
            people: item.xkrs,
            schedule,
        });
    }
    Ok(courses)
}

/// `json_str` 为 [`super::fetch::class_table`] 返回的数据
pub fn class_table_extra(json_str: &str) -> Result<Vec<ExtraCourse>, crate::Error<TokenExpired>> {
    let json = crate::hdjw::parse::hdjw_response(json_str)?;
    let raw_data = match json.get("count").and_then(|c| c.as_u64()) {
        None => return Err(parse_err("无法解析无课程表数据", json_str)),
        Some(0) => return Ok(vec![]), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => serde_json::from_value::<Vec<RawCourseInfo>>(json["data"].clone())
            .parse_err(json_str)?,
    };
    let mut courses = Vec::with_capacity(raw_data.len());
    // 无课表课程不含 sktime 字段
    for item in raw_data.into_iter().filter(|item| item.sktime.is_none()) {
        courses.push(ExtraCourse {
            // 教务系统可能会返回空格或制表符开头/结尾的课程名
            course_name: item.kc_mc.trim().to_string(),
            course_id: item.kch,
            course_type: item.kcxz,
            class_name: item.kt_mc,
            area: item.skxqmc,
            // 教务系统可能会返回空格开头或结尾
            teacher: item
                .jg0101mc
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
            credit: item.xf,
            extra: item.fz_mc,
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

    #[test]
    fn test_class_table_mixed() -> TestResult<()> {
        // 新格式 sktime + 混合响应（8 门有 sktime + 2 门缺）
        let courses = class_table(include_str!("test_data/xskb_list_mixed.json"))?;

        // 缺 sktime 的「创新创业」「数据结构与算法」（第一条）应被过滤掉
        assert_eq!(courses.len(), 8);

        // 形势与政策(5) -- 简单：单天单时间段
        let test_course_1 = find_course(&courses, "形势与政策(5)");
        assert_eq!(test_course_1.course_id, "GE01159");
        assert_eq!(test_course_1.credit, 0.25);
        assert_eq!(test_course_1.course_type, "通识必修");
        assert_eq!(test_course_1.class_name, "人工智能2301-02");
        assert_eq!(test_course_1.area, "南校区");
        assert_eq!(test_course_1.teacher, Some("教师1".to_string()));
        assert_eq!(test_course_1.extra, None);
        assert_eq!(test_course_1.people, 69);
        assert_eq!(test_course_1.schedule.len(), 2);
        let course_1_weeks: HashSet<u8> = test_course_1.schedule.iter().map(|s| s.week).collect();
        assert_eq!(course_1_weeks, HashSet::from([9, 10]));
        for s in &test_course_1.schedule {
            assert_eq!(s.day, 1);
            assert_eq!(s.place, "复403");
            assert_eq!(sorted(s.time.clone()), vec![7, 8]);
        }

        // 数据结构与算法 -- (单周) 后缀、逗号枚举与区间混合的周次
        let test_course_2 = find_course(&courses, "数据结构与算法");
        assert_eq!(test_course_2.course_id, "RO04010");
        assert_eq!(test_course_2.teacher, Some("教师6,教师7".to_string()));
        assert_eq!(test_course_2.schedule.len(), 23);

        // 星期四7、8节{ 1,3,5,7,9,11,13,15周}(单周) → 8 entries
        let day4: Vec<_> = test_course_2
            .schedule
            .iter()
            .filter(|s| s.day == 4)
            .collect();
        assert_eq!(day4.len(), 8);
        let day4_weeks: HashSet<u8> = day4.iter().map(|s| s.week).collect();
        assert_eq!(day4_weeks, (1..=15).step_by(2).collect());
        for s in &day4 {
            assert_eq!(s.place, "复309");
            assert_eq!(sorted(s.time.clone()), vec![7, 8]);
        }

        // 星期一5、6节{ 1-2,4-16周}(全周) → 15 entries
        let day1: Vec<_> = test_course_2
            .schedule
            .iter()
            .filter(|s| s.day == 1)
            .collect();
        assert_eq!(day1.len(), 15);
        let day1_weeks: HashSet<u8> = day1.iter().map(|s| s.week).collect();
        assert_eq!(day1_weeks, (1..=2).chain(4..=16).collect());
        for s in &day1 {
            assert_eq!(s.place, "复309");
            assert_eq!(sorted(s.time.clone()), vec![5, 6]);
        }

        // 普通物理A（1） -- 星期六、节次区间、fz_mc 备注、多段多地点
        let test_course_3 = find_course(&courses, "普通物理A（1）");
        assert_eq!(test_course_3.course_id, "GE03005");
        assert_eq!(test_course_3.extra, Some("普通物理同一课堂".to_string()));
        assert_eq!(test_course_3.schedule.len(), 5);
        for s in &test_course_3.schedule {
            assert_eq!(s.day, 6);
            assert_eq!(sorted(s.time.clone()), vec![2, 3, 4]);
            if s.week == 12 {
                assert_eq!(s.place, "东111");
            } else {
                assert!(HashSet::from([4, 7, 8, 11]).contains(&s.week));
                assert_eq!(s.place, "研C202");
            }
        }

        // 计算机视觉与模式识别 -- 4 个时间段
        let test_course_4 = find_course(&courses, "计算机视觉与模式识别");
        assert_eq!(test_course_4.schedule.len(), 23);
        let day6: Vec<_> = test_course_4
            .schedule
            .iter()
            .filter(|s| s.day == 6)
            .collect();
        assert_eq!(day6.len(), 2);
        for s in &day6 {
            match s.week {
                3 => assert_eq!(sorted(s.time.clone()), vec![7, 8]),
                7 => assert_eq!(sorted(s.time.clone()), vec![1, 2]),
                week => panic!("意外的周次 {}", week),
            }
        }

        Ok(())
    }

    #[test]
    fn test_class_table_week0() -> TestResult<()> {
        // 第 0 周、星期日、地点为「无」、实践环节课带 sktime
        let courses = class_table(include_str!("test_data/xskb_list_week0.json"))?;

        // 缺 sktime 的「普通物理实验AⅡ」应被过滤掉
        assert_eq!(courses.len(), 12);

        // 机械制图A -- 含第 0 周
        let test_course_1 = find_course(&courses, "机械制图A");
        assert_eq!(test_course_1.schedule.len(), 28);
        let day1: Vec<_> = test_course_1
            .schedule
            .iter()
            .filter(|s| s.day == 1)
            .collect();
        assert_eq!(day1.len(), 14);
        let day1_weeks: HashSet<u8> = day1.iter().map(|s| s.week).collect();
        assert_eq!(day1_weeks, (0..=2).chain(6..=16).collect());
        for s in &day1 {
            assert_eq!(s.place, "研B106");
            assert_eq!(sorted(s.time.clone()), vec![5, 6]);
        }
        let day4: Vec<_> = test_course_1
            .schedule
            .iter()
            .filter(|s| s.day == 4)
            .collect();
        assert_eq!(day4.len(), 14);
        let day4_weeks: HashSet<u8> = day4.iter().map(|s| s.week).collect();
        assert_eq!(
            day4_weeks,
            HashSet::from([0, 1, 3, 6])
                .union(&(7..=16).collect())
                .copied()
                .collect()
        );
        for s in &day4 {
            assert_eq!(s.place, "研B107");
            assert_eq!(sorted(s.time.clone()), vec![3, 4]);
        }

        // 机械工程导论 -- 含星期日、地点为「无」
        let test_course_2 = find_course(&courses, "机械工程导论");
        assert_eq!(test_course_2.schedule.len(), 16);
        let day7: Vec<_> = test_course_2
            .schedule
            .iter()
            .filter(|s| s.day == 7)
            .collect();
        assert_eq!(day7.len(), 8);
        let day7_weeks: HashSet<u8> = day7.iter().map(|s| s.week).collect();
        assert_eq!(day7_weeks, (7..=14).collect());
        for s in &day7 {
            assert_eq!(s.place, "无");
            assert_eq!(sorted(s.time.clone()), vec![1, 2, 3, 4]);
        }

        // 体育Ⅲ -- 地点为「无」、fz_mc 备注
        let test_course_3 = find_course(&courses, "体育Ⅲ");
        assert_eq!(
            test_course_3.extra,
            Some("周一34节田径（男）【南校区田径场】".to_string())
        );
        assert_eq!(test_course_3.schedule.len(), 15);
        for s in &test_course_3.schedule {
            assert_eq!(s.place, "无");
        }

        // 电工电子训练A -- 实践环节课也带 sktime；两个时间段同周同天同地点，节次应合并
        let test_course_4 = find_course(&courses, "电工电子训练A");
        assert_eq!(test_course_4.course_type, "实践环节");
        assert_eq!(test_course_4.schedule.len(), 8);
        let course_4_weeks: HashSet<u8> = test_course_4.schedule.iter().map(|s| s.week).collect();
        assert_eq!(course_4_weeks, (9..=16).collect());
        for s in &test_course_4.schedule {
            assert_eq!(s.day, 5);
            assert_eq!(s.place, "无");
            assert_eq!(sorted(s.time.clone()), vec![1, 2, 3, 4, 5, 6, 7, 8]);
        }

        Ok(())
    }

    #[test]
    fn test_class_table_all_no_schedule() -> TestResult<()> {
        // 全部课程缺 sktime（纯实践学期），有课表结果应为空且不报错
        let courses = class_table(include_str!("test_data/xskb_list_all_no_schedule.json"))?;
        assert_eq!(courses.len(), 0);

        Ok(())
    }

    #[test]
    fn test_class_table_extra_mixed() -> TestResult<()> {
        // 混合响应中缺 sktime 的课程应进入无课表结果
        let courses = class_table_extra(include_str!("test_data/xskb_list_mixed.json"))?;

        assert_eq!(courses.len(), 2);

        let course_1 = &courses[0];
        // 课程名带尾随制表符，应被去除
        assert_eq!(course_1.course_name, "创新创业");
        assert_eq!(course_1.course_id, "GE09083");
        assert_eq!(course_1.course_type, "实践环节");
        assert_eq!(course_1.class_name, "人工智能2301-02");
        assert_eq!(course_1.area, "南校区");
        assert_eq!(course_1.teacher, "教师3");
        assert_eq!(course_1.credit, 2.0);
        assert_eq!(course_1.extra, None);
        assert_eq!(course_1.people, 69);

        // 同名课程「数据结构与算法」在无课表与有课表结果中各出现一次（不同教学班）
        let course_2 = &courses[1];
        assert_eq!(course_2.course_name, "数据结构与算法");
        assert_eq!(course_2.teacher, "教师4,教师5");

        Ok(())
    }

    #[test]
    fn test_class_table_extra_all_no_schedule() -> TestResult<()> {
        let courses = class_table_extra(include_str!("test_data/xskb_list_all_no_schedule.json"))?;

        assert_eq!(courses.len(), 4);

        let course = &courses[0];
        assert_eq!(course.course_id, "BA10048");
        assert_eq!(course.course_name, "审计模拟实习");
        assert_eq!(course.course_type, "实践环节");
        assert_eq!(course.class_name, "ACCA2301班,会计[2301-2302]班");
        assert_eq!(course.area, "财院校区");
        assert_eq!(course.teacher, "教师1");
        assert_eq!(course.credit, 1.0);
        assert_eq!(course.extra, None);
        assert_eq!(course.people, 60);

        Ok(())
    }

    #[test]
    fn test_class_table_extra_week0() -> TestResult<()> {
        let courses = class_table_extra(include_str!("test_data/xskb_list_week0.json"))?;

        assert_eq!(courses.len(), 1);
        assert_eq!(courses[0].course_name, "普通物理实验AⅡ");
        assert_eq!(courses[0].teacher, "教师20,教师21,教师22");

        Ok(())
    }
}
