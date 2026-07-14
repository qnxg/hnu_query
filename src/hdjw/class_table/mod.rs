mod fetch;
mod parse;

use crate::{
    hdjw::{error::TokenExpired, login::HdjwToken},
    utils::obs::{fetch_time, parse_time, traced},
};
use serde::{Deserialize, Serialize};

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
#[derive(Deserialize, Serialize, Debug, Clone)]
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
#[traced(subsystem = "hdjw", skip(hdjw_token))]
pub async fn get_class_table(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let raw_data = fetch_time!(fetch::class_table(hdjw_token, xn, xq).await)?;
    parse_time!(parse::class_table(&raw_data))
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
#[traced(subsystem = "hdjw", skip(hdjw_token))]
pub async fn get_class_table_extra(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<ExtraCourse>, crate::Error<TokenExpired>> {
    let json_str = fetch_time!(fetch::class_table_extra(hdjw_token, xn, xq).await)?;
    parse_time!(parse::class_table_extra(&json_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hdjw::test::get_hdjw_token;
    use crate::test::{TEST_XN, TEST_XQ, TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_classtable() -> TestResult<()> {
        let hdjw_token = get_hdjw_token().await?;
        let classtable = get_class_table(&hdjw_token, *TEST_XN, *TEST_XQ).await?;
        println!("{:#?}", classtable);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_class_table_extra() -> TestResult<()> {
        let hdjw_token = get_hdjw_token().await?;
        let extra_courses = get_class_table_extra(&hdjw_token, *TEST_XN, *TEST_XQ).await?;
        println!("{:#?}", extra_courses);
        Ok(())
    }
}
