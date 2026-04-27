use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    lab::login::LabToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, convert::Infallible};

const LAB_SCORE_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentLabScore";
const VIRTUAL_LAB_SCORE_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentFZLabScore";
const LAB_SCORE_STRUCTURE_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/GetLabScoreStructure";
const LAB_SCORE_DETAIL_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/ShowScore";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreItem {
    /// 出勤情况
    pub AttendanceName: String,
    /// 实验名称
    pub LabName: String,
    /// 实验成绩，没有成绩的话是空字符串
    pub LabScore: String,
    /// 实验id
    pub LabID: String,
    /// 上课地点，这个字段只是用来判断是否为虚拟实验的
    pub ClassRoom: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreDetailItem {
    /// 对应的成绩结构id
    pub LabScoreStructureID: i32,
    /// 对应的实验id
    pub LabID: i32,
    /// 分数
    pub LabStructureScore: Option<f64>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreStructureItem {
    /// 成绩结构id
    pub LabScoreStructureID: i32,
    /// 成绩结构名称
    pub LabScoreStructureName: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct VirtualLabScoreItem {
    /// 实验名称
    pub LabName: String,
    /// 实验成绩，没有成绩的话是空字符串
    pub LabScore: String,
}

/// 获取某个课程的实验成绩
///
/// 这里面应该是包含了虚拟实验的。但是貌似专门的虚拟实验的成绩接口能得到最新成绩
pub async fn raw_lab_score_data(
    lab_token: &LabToken,
    course_id: &str,
    semester_id: &str,
) -> Result<Vec<LabScoreItem>, crate::Error<Infallible>> {
    let headers = lab_token.headers().clone();
    let mut form_data = HashMap::new();
    form_data.insert("page", "1");
    form_data.insert("rows", "15");
    form_data.insert("SemID", semester_id);
    form_data.insert("CourseID", course_id);
    form_data.insert("UserID", lab_token.stu_id());
    let json_str = client
        .post(LAB_SCORE_URL)
        .form(&form_data)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("rows")
        .map(|v| serde_json::from_value::<Vec<LabScoreItem>>(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}

pub async fn raw_lab_score_structure_data(
    lab_token: &LabToken,
    course_id: &str,
) -> Result<Vec<LabScoreStructureItem>, crate::Error<Infallible>> {
    let headers = lab_token.headers().clone();
    let json_str = client
        .get(format!(
            "{}?CourseID={}",
            LAB_SCORE_STRUCTURE_URL, course_id
        ))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("Data")
        .map(|v| {
            serde_json::from_value::<Vec<LabScoreStructureItem>>(v.clone()).parse_err(&json_str)
        })
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}

pub async fn raw_lab_score_detail_data(
    lab_token: &LabToken,
    course_id: &str,
) -> Result<Vec<LabScoreDetailItem>, crate::Error<Infallible>> {
    let headers = lab_token.headers().clone();
    let json_str = client
        .get(format!(
            "{}?CourseID={}&StudentID={}",
            LAB_SCORE_DETAIL_URL,
            course_id,
            lab_token.stu_id()
        ))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("Data")
        .and_then(|v| v.get("Lablist"))
        .map(|v| serde_json::from_value::<Vec<LabScoreDetailItem>>(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}

/// 获取虚拟实验的成绩
///
/// 虚拟实验的接口有点奇怪，经过测试，无论学期和课程id怎么给，都会返回一个学期的虚拟实验的成绩
pub async fn raw_virtual_lab_score_data(
    lab_token: &LabToken,
) -> Result<Vec<VirtualLabScoreItem>, crate::Error<Infallible>> {
    let headers = lab_token.headers().clone();
    let mut form_data = HashMap::new();
    form_data.insert("page", "1");
    form_data.insert("rows", "15");
    // 既然怎么给都无所谓，就随便给
    form_data.insert("SemID", "0");
    form_data.insert("CourseID", "0");
    form_data.insert("UserID", lab_token.stu_id());
    let json_str = client
        .post(VIRTUAL_LAB_SCORE_URL)
        .form(&form_data)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("rows")
        .map(|v| serde_json::from_value::<Vec<VirtualLabScoreItem>>(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}
