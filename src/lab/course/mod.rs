mod fetch;
mod parse;

use crate::lab::login::LabToken;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 大物实验平台的课程信息
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Course {
    /// 课程名称
    pub name: String,
    /// 课程成绩
    ///
    /// 为 None 说明暂时没有成绩
    pub score: Option<String>,
    /// 课程id
    pub id: String,
}

/// 获取课程列表
///
/// # Arguments
///
/// - `lab_token`: 大物实验平台的令牌，可以通过 [LabToken::acquire_by_login] 获取
/// - `semester_id`: 学期id，需要通过 [`crate::lab::get_semester`] 获取
///
/// # Returns
///
/// 返回课程列表
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(lab_token), fields(subsystem = "lab"), err)
)]
pub async fn get_course_list(
    lab_token: &LabToken,
    semester_id: &str,
) -> Result<Vec<Course>, crate::Error<Infallible>> {
    let json_str = fetch::course_list(lab_token, semester_id).await?;
    parse::course_list(&json_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        lab::test::{TEST_SEMESTER_ID, get_lab_token},
        test::TestResult,
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_course_list() -> TestResult<()> {
        let lab_token = get_lab_token().await?;
        let course_list = get_course_list(&lab_token, TEST_SEMESTER_ID).await?;
        println!("{:#?}", course_list);
        Ok(())
    }
}
