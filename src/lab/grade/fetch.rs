use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    lab::login::LabToken,
    utils::client,
};
use std::{collections::HashMap, convert::Infallible};

const LAB_SCORE_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentLabScore";
const VIRTUAL_LAB_SCORE_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentFZLabScore";
const LAB_SCORE_STRUCTURE_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/GetLabScoreStructure";
const LAB_SCORE_DETAIL_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/ShowScore";

/// 获取某个课程的实验成绩
///
/// 这里面应该是包含了虚拟实验的。但是貌似专门的虚拟实验的成绩接口能得到最新成绩
pub async fn lab_grade(
    lab_token: &LabToken,
    course_id: &str,
    semester_id: &str,
) -> Result<String, crate::Error<Infallible>> {
    let mut form_data = HashMap::new();
    form_data.insert("page", "1");
    form_data.insert("rows", "15");
    form_data.insert("SemID", semester_id);
    form_data.insert("CourseID", course_id);
    form_data.insert("UserID", lab_token.stu_id());
    client
        .post(LAB_SCORE_URL)
        .form(&form_data)
        .headers(lab_token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn lab_grade_structure(
    lab_token: &LabToken,
    course_id: &str,
) -> Result<String, crate::Error<Infallible>> {
    client
        .get(format!(
            "{}?CourseID={}",
            LAB_SCORE_STRUCTURE_URL, course_id
        ))
        .headers(lab_token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn lab_grade_detail(
    lab_token: &LabToken,
    course_id: &str,
) -> Result<String, crate::Error<Infallible>> {
    client
        .get(format!(
            "{}?CourseID={}&StudentID={}",
            LAB_SCORE_DETAIL_URL,
            course_id,
            lab_token.stu_id()
        ))
        .headers(lab_token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}

/// 获取虚拟实验的成绩
///
/// 虚拟实验的接口有点奇怪，经过测试，无论学期和课程id怎么给，都会返回一个学期的虚拟实验的成绩
pub async fn virtual_lab_grade(lab_token: &LabToken) -> Result<String, crate::Error<Infallible>> {
    let headers = lab_token.headers().clone();
    let mut form_data = HashMap::new();
    form_data.insert("page", "1");
    form_data.insert("rows", "15");
    // 既然怎么给都无所谓，就随便给
    form_data.insert("SemID", "0");
    form_data.insert("CourseID", "0");
    form_data.insert("UserID", lab_token.stu_id());
    client
        .post(VIRTUAL_LAB_SCORE_URL)
        .form(&form_data)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
