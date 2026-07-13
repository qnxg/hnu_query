mod fetch;
mod parse;

use crate::yjsxt::{error::TokenExpired, login::YjsxtToken};
use serde::{Deserialize, Serialize};

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

/// 获取课表信息
///
/// # Arguments
///
/// - `yjsxt_token` - 研究生系统的令牌，可以通过 [YjsxtToken::acquire_by_cas_login] 获取
/// - `semester_id` - 学期id，可以通过 [get_semester](super::get_semester) 获取
///
/// # Returns
///
/// 返回所选课程的列表
///
/// # Errors
///
/// 如果提供的 `yjsxt_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [YjsxtToken]
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(yjsxt_token), fields(subsystem = "yjsxt"), err)
)]
pub async fn get_class_table(
    yjsxt_token: &YjsxtToken,
    semester_id: u16,
) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let json_str = fetch::class_table(yjsxt_token, semester_id).await?;
    parse::class_table(&json_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::TestResult,
        yjsxt::test::{TEST_YJSXT_SEMESTER_ID, get_yjsxt_token},
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_class_table() -> TestResult<()> {
        let yjsxt_token = get_yjsxt_token().await?;
        let class_table = get_class_table(&yjsxt_token, *TEST_YJSXT_SEMESTER_ID).await?;
        println!("{:#?}", class_table);
        Ok(())
    }
}
