mod raw;

use crate::{
    error::{MapParseErr, parse_err_with_reason},
    hdjw::{
        class_table::raw::{raw_class_table_data, raw_class_table_extra_data},
        error::TokenExpired,
        login::HdjwToken,
    },
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 课程信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Course {
    /// 课程名称
    pub course_name: String,
    /// 课程代码
    pub course_id: String,
    /// 课程类型
    pub course_type: String,
    /// 上课班级
    pub class_name: String,
    /// 上课校区
    pub area: String,
    /// 授课教师
    ///
    /// 可能会有多位教师，用 `,` 分隔，还有可能没有教师
    pub teacher: Option<String>,
    /// 学分
    pub credit: f32,
    /// 额外备注信息
    pub extra: Option<String>,
    /// 上课人数
    pub people: u16,
    /// 课程的时间地点安排
    ///
    /// Vec 内的元素不保证有序，不保证永远都是一个顺序。保证不重。
    ///
    /// 不存在 `week`、`day`、`place` 都相同的 `CourseSchedule`（假如有，他们两个的 `time` 必然可以合并）
    ///
    /// 可能会出现 `week`、`day` 相同但是 `place` 不同，这意味着这一天要去不同的地点上课
    pub schedule: Vec<CourseSchedule>,
}

/// 课程的时间地点安排
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CourseSchedule {
    /// 第几周上课，比如 `week` 为 16 就表示第 16 周上课
    ///
    /// 通常这个取值范围为 [1, 16] 或是 [1, 18]，
    /// 据说也出现过一学期有 19 周的情况。
    /// 秋季学期往往还会存在第 0 周，用于新生提前开学。
    pub week: u8,
    /// 周几上课，比如 `day` 为 1 就表示周一上课，`day` 为 7 就表示周日上课
    ///
    /// 注意，湖大规定，一周的第一天是周日。比如今天是第 2 周周六，
    /// 那么明天就是第 3 周周日，第 2 周周一的前一天才是第 2 周周日
    pub day: u8,
    /// 上课地点
    pub place: String,
    /// 上课的节次。
    ///
    /// Vec 内的元素表示的是小节次。参考 `docs/hdjw/time.md` 中的 `节次` 字段
    ///
    /// Vec 内的元素不保证有序，不保证永远都是一个顺序。保证不重。
    pub time: Vec<u8>,
}

/// 无课表课程信息
///
/// 相比于 `Course`，仅少了 `schedule` 字段
#[derive(Deserialize, Debug)]
pub struct ExtraCourse {
    /// 课程名称
    pub course_name: String,
    /// 课程代码
    pub course_id: String,
    /// 课程类型
    pub course_type: String,
    /// 上课班级
    pub class_name: String,
    /// 上课校区
    pub area: String,
    /// 授课教师
    ///
    /// 可能会有多位教师，用 `,` 分隔
    pub teacher: String,
    /// 学分
    pub credit: f32,
    /// 额外备注信息
    pub extra: Option<String>,
    /// 上课人数
    pub people: u16,
}

/// 获取课表信息
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌，可以通过 [HdjwToken::acquire_by_cas_login] 获取
/// - `xn`: 学年
/// - `xq`: 学期
///
/// # Returns
///
/// 返回所选课程的列表
///
/// # Errors
///
/// 如果提供的 `hdjw_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [HdjwToken]
#[expect(clippy::too_many_lines, reason = "REFACTOR ME")]
pub async fn get_class_table(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let raw_data = raw_class_table_data(hdjw_token, xn, xq).await?;
    let mut courses = Vec::with_capacity(raw_data.len());
    let re = Regex::new(r"周(.)第(.*)节.*\{第(.*)周\}").expect("创建正则表达式失败");
    for item in raw_data {
        let places = item.skddmc.split(';').collect::<Vec<_>>();
        let detail_times = item.sktime.split(';');
        // 第几周+周几+地点作为 key，节次作为 value，进行去重
        let mut schedule = HashMap::new();
        for (i, time) in detail_times.into_iter().enumerate() {
            let caps = re
                .captures(time)
                .ok_or(parse_err_with_reason(&item.sktime, "上课时间: day"))?;
            let day = match caps.get(1).and_then(|v| v.as_str().chars().next()).ok_or(
                parse_err_with_reason(&item.sktime, "上课时间: day: 没有匹配到星期字符"),
            )? {
                '一' => 1,
                '二' => 2,
                '三' => 3,
                '四' => 4,
                '五' => 5,
                '六' => 6,
                '日' | '七' => 7,
                day => {
                    return Err(parse_err_with_reason(
                        &item.sktime,
                        &format!("上课时间: day: 未知的星期字符: {}", day),
                    ));
                }
            };
            // 节次信息首先由 、分割，分割出来的每个部分即可能是一个单个数字，有可能是一个区间范围（由 - 连接）
            let mut time_list = HashSet::new();
            for time_range in caps
                .get(2)
                .ok_or(parse_err_with_reason(&item.sktime, "上课时间: time"))?
                .as_str()
                .split('、')
                .collect::<Vec<_>>()
            {
                let parts = time_range.split('-').collect::<Vec<_>>();
                let time_l = parts
                    .first()
                    .and_then(|v| v.parse::<u8>().ok())
                    .ok_or(parse_err_with_reason(&item.sktime, "上课时间: time"))?;
                let time_r = match parts.get(1) {
                    Some(v) => v
                        .parse::<u8>()
                        .parse_err_with_reason(&item.sktime, "上课时间: time")?,
                    None => time_l,
                };
                time_list.extend(time_l..=time_r);
            }
            // 周次信息首先由 , 分割，分割出来的每个部分即可能是一个单个数字，有可能是一个区间范围（由 - 连接）
            let mut week_list = HashSet::new();
            for week_range in caps
                .get(3)
                .ok_or(parse_err_with_reason(&item.sktime, "上课时间: week"))?
                .as_str()
                .split(',')
                .collect::<Vec<_>>()
            {
                let parts = week_range.split('-').collect::<Vec<_>>();
                let week_l = parts
                    .first()
                    .and_then(|v| v.parse::<u8>().ok())
                    .ok_or(parse_err_with_reason(&item.sktime, "上课时间: week"))?;
                let week_r = match parts.get(1) {
                    Some(v) => v
                        .parse::<u8>()
                        .parse_err_with_reason(&item.sktime, "上课时间: week")?,
                    None => week_l,
                };
                week_list.extend(week_l..=week_r);
            }
            let place = places
                .get(i)
                .ok_or(parse_err_with_reason(&item.skddmc, "上课地点"))?;
            week_list.iter().for_each(|&week| {
                schedule
                    .entry((week, day, place.to_string()))
                    .or_insert_with(HashSet::<u8>::new)
                    .extend(time_list.iter());
            });
        }
        let temp = Course {
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
            schedule: schedule
                .into_iter()
                .map(|((week, day, place), time)| CourseSchedule {
                    week,
                    day,
                    place,
                    time: time.into_iter().collect(),
                })
                .collect(),
        };
        courses.push(temp);
    }

    Ok(courses)
}

/// 获取无课表课程信息
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌，可以通过 [HdjwToken::acquire_by_cas_login] 获取
/// - `xn`: 学年
/// - `xq`: 学期
///
/// # Returns
///
/// 返回无课表课程列表
///
/// # Errors
///
/// 如果提供的 `hdjw_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [HdjwToken]
pub async fn get_class_table_extra(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<ExtraCourse>, crate::Error<TokenExpired>> {
    let raw_data = raw_class_table_extra_data(hdjw_token, xn, xq).await?;
    let mut courses = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let temp = ExtraCourse {
            course_name: item.kc_mc,
            course_id: item.kch,
            course_type: item.kcxz,
            class_name: item.kt_mc,
            area: item.skxqmc,
            teacher: item.jg0101mc,
            credit: item.xf,
            extra: item.fzmc,
            people: item.xkrs,
        };
        courses.push(temp);
    }
    Ok(courses)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hdjw::test::get_hdjw_token;
    use crate::test::{TEST_XN, TEST_XQ};

    #[tokio::test]
    #[ignore]
    async fn test_get_classtable() {
        let hdjw_token = get_hdjw_token().await.unwrap();
        let classtable = get_class_table(&hdjw_token, *TEST_XN, *TEST_XQ)
            .await
            .unwrap();
        println!("{:#?}", classtable);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_class_table_extra() {
        let hdjw_token = get_hdjw_token().await.unwrap();
        let extra_courses = get_class_table_extra(&hdjw_token, *TEST_XN, *TEST_XQ)
            .await
            .unwrap();
        println!("{:#?}", extra_courses);
    }
}
