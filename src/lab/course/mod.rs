mod raw;

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
/// # Parameters
///
/// - `lab_token`: 大物实验平台的令牌，可以通过 [LabToken::acquire_by_login] 获取
/// - `semester_id`: 学期id，需要通过 [`crate::lab::get_semester`] 获取
///
/// # Returns
///
/// 返回课程列表
pub async fn get_course_list(
    lab_token: &LabToken,
    semester_id: &str,
) -> Result<Vec<Course>, crate::Error<Infallible>> {
    let raw_data = raw::raw_course_list_data(lab_token, semester_id).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        res.push(Course {
            name: item.CourseName,
            score: if item.CourseFinalScore.is_empty() {
                None
            } else {
                Some(item.CourseFinalScore)
            },
            id: item.CourseID,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lab::test::{TEST_SEMESTER_ID, get_lab_token};

    #[tokio::test]
    #[ignore]
    async fn test_get_course_list() {
        let lab_token = get_lab_token().await.unwrap();
        let course_list = get_course_list(&lab_token, TEST_SEMESTER_ID).await.unwrap();
        println!("{:#?}", course_list);
    }
}
