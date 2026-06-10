mod parse;
mod raw;

use crate::hdjw::{error::TokenExpired, login::HdjwToken};
use serde::{Deserialize, Serialize};

/// 课程成绩
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Grade {
    /// 课程代码
    pub course_id: String,
    /// 课程名称
    pub course_name: String,
    /// 学分
    pub credit: f32,
    /// 课程性质1，如`必修`、`选修`
    pub course_type1: Option<String>,
    /// 课程性质2，如`通识必修`、`专业核心`等
    pub course_type2: String,
    /// 该门课程获得的绩点
    pub gpa: Option<f32>,
    /// 该门课程获得的分数
    pub score: f64,
    /// 成绩标识
    ///
    /// 如果成绩正常则为 `None`，否则为类似 `缓考`、`重修` 等标识
    ///
    /// 如果有门课程有缓考和重修，那么该课程会有两门成绩，一门是全校统一考试时成绩
    /// ，该成绩会被标上成绩标识，成绩为 0 分；另一门成绩是补考的成绩。
    pub grade_tag: Option<String>,
    /// 成绩类型，如 `主修`、`辅修` 等
    pub grade_type: String,
    /// 猜测应该是成绩独一无二的 id，用于获取成绩详情
    pub jx0404id: Option<String>,
}

/// 获取课程成绩
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌，可以通过 [HdjwToken::acquire_by_cas_login] 获取
/// - `xn`: 学年
/// - `xq`: 学期
///
/// # Returns
///
/// 返回一个包含给定学年学期的课程成绩的列表
///
/// # Errors
///
/// 如果提供的 `hdjw_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [HdjwToken]
pub async fn get_grade(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<Grade>, crate::Error<TokenExpired>> {
    let raw_data = raw::get_cjcx_list(hdjw_token, xn, xq).await?;
    parse::grade(raw_data)
}

/// 课程成绩的组成部分
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GradeDetailItem {
    /// 成绩组成名称
    pub name: String,
    /// 该成绩组成所占的分数
    // TODO 进一步解析成浮点数
    pub score: String,
    /// 该成绩组成所占的百分比，形如 `50%`
    // TODO 进一步解析成整数
    pub percentage: String,
}

/// 获取课程成绩详情
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌，可以通过 [HdjwToken::acquire_by_cas_login] 获取
/// - `jx0404id`: 通过 [get_grade] 获得的 [Grade::jx0404id]
///
/// # Returns
///
/// Vec 内的每个元素表示该课程成绩的一个组成部分，一个课程成绩由多个组成部分构成
///
/// # Errors
///
/// 如果提供的 `hdjw_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [HdjwToken]
pub async fn get_grade_detail(
    hdjw_token: &HdjwToken,
    jx0404id: &str,
) -> Result<Vec<GradeDetailItem>, crate::Error<TokenExpired>> {
    let raw_data = raw::get_pscj_list(hdjw_token, jx0404id).await?;
    parse::grade_detail(&raw_data)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        hdjw::test::{TEST_HDJW_JX0404ID, get_hdjw_token},
        test::{TEST_XN, TEST_XQ, TestResult},
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_grade() -> TestResult<()> {
        let hdjw_token = get_hdjw_token().await?;
        let grade = get_grade(&hdjw_token, *TEST_XN, *TEST_XQ).await?;
        println!("{:#?}", grade);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_grade_detail() -> TestResult<()> {
        let hdjw_token = get_hdjw_token().await?;
        let grade_detail = get_grade_detail(&hdjw_token, TEST_HDJW_JX0404ID).await?;
        println!("{:#?}", grade_detail);
        Ok(())
    }
}
