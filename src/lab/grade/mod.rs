mod fetch;
mod parse;

use crate::lab::login::LabToken;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio::try_join;

/// 实验成绩
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LabGrade {
    /// 实验名称
    pub lab_name: String,
    /// 实验成绩
    pub score: String,
    /// 出勤情况
    pub attendance: Option<String>,
    /// 成绩的具体组成
    pub details: Vec<LabGradeDetailItem>,
}

/// 实验成绩的具体组成项
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LabGradeDetailItem {
    /// 成绩组成名称
    pub name: String,
    /// 分数
    ///
    /// 为 None 说明没有成绩
    pub score: Option<f64>,
}

/// 获取某门课程下的实验成绩
///
/// # Parameters
///
/// - `lab_token`: 大物实验平台的令牌，可以通过 [LabToken::acquire_by_login] 获取
/// - `course_id`: 课程id，通过 [`crate::lab::get_course_list`] 获取
/// - `semester_id`: 学期id，通过 [`crate::lab::get_semester`] 获取
///
/// # Returns
///
/// 返回实验成绩列表
pub async fn get_lab_grade(
    lab_token: &LabToken,
    course_id: &str,
    semester_id: &str,
) -> Result<Vec<LabGrade>, crate::Error<Infallible>> {
    let (lab_grade_str, lab_grade_detail_str, lab_grade_structure_str) = try_join!(
        fetch::lab_grade(lab_token, course_id, semester_id),
        fetch::lab_grade_detail(lab_token, course_id),
        fetch::lab_grade_structure(lab_token, course_id),
    )?;
    parse::lab_grade(
        &lab_grade_str,
        &lab_grade_detail_str,
        &lab_grade_structure_str,
    )
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct VirtualLabGrade {
    /// 实验名称
    pub lab_name: String,
    /// 实验成绩
    ///
    /// 为 None 说明没有成绩
    pub score: Option<String>,
}

/// 获取虚拟实验成绩
///
/// # Parameters
///
/// - `lab_token`: 大物实验平台的令牌，可以通过 [LabToken::acquire_by_login] 获取
///
/// # Returns
///
/// 返回虚拟实验成绩列表
pub async fn get_virtual_lab_grade(
    lab_token: &LabToken,
) -> Result<Vec<VirtualLabGrade>, crate::Error<Infallible>> {
    let json_str = fetch::virtual_lab_grade(lab_token).await?;
    parse::virtual_lab_grade(&json_str)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        lab::test::{TEST_COURSE_ID, TEST_SEMESTER_ID, get_lab_token},
        test::TestResult,
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_lab_grade() -> TestResult<()> {
        let lab_token = get_lab_token().await?;
        let grade = get_lab_grade(&lab_token, TEST_COURSE_ID, TEST_SEMESTER_ID).await?;
        println!("{:#?}", grade);
        Ok(())
    }
}
