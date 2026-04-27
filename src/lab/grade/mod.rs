mod raw;

use crate::{
    error::MapParseErr,
    lab::{grade::raw::raw_virtual_lab_score_data, login::LabToken},
};
use raw::{raw_lab_score_data, raw_lab_score_detail_data, raw_lab_score_structure_data};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible};
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
    let (lab_score, lab_score_detail, lab_score_structure) = try_join!(
        raw_lab_score_data(lab_token, course_id, semester_id),
        raw_lab_score_detail_data(lab_token, course_id),
        raw_lab_score_structure_data(lab_token, course_id),
    )?;
    let score_structure_map: HashMap<i32, String> = lab_score_structure
        .into_iter()
        .map(|item| (item.LabScoreStructureID, item.LabScoreStructureName))
        .collect();
    let mut lab_map: HashMap<i32, usize> = HashMap::new();
    let mut res = Vec::new();
    // 过滤还没有成绩的实验和虚拟实验
    for item in lab_score
        .into_iter()
        .filter(|i| !i.LabScore.is_empty() && !i.ClassRoom.contains("虚拟"))
    {
        let lab_id = item.LabID.parse::<i32>().parse_err(&item.LabID)?;
        res.push(LabGrade {
            lab_name: item.LabName,
            score: item.LabScore,
            attendance: if item.AttendanceName.is_empty() {
                None
            } else {
                Some(item.AttendanceName)
            },
            details: Vec::new(),
        });
        lab_map.insert(lab_id, res.len() - 1);
    }
    for item in lab_score_detail
        .into_iter()
        .filter(|i| i.LabStructureScore.is_some())
    {
        if let Some(index) = lab_map.get(&item.LabID)
            && let Some(structure_name) = score_structure_map.get(&item.LabScoreStructureID)
        {
            // labs 和 lab_map 保证了一一对应关系，这里不会有 None
            let lab = res.get_mut(*index).expect("根据实验 id 获得的 index 无效");
            lab.details.push(LabGradeDetailItem {
                name: structure_name.clone(),
                score: item.LabStructureScore,
            });
        }
    }
    Ok(res)
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
    let spider_res = raw_virtual_lab_score_data(lab_token).await?;
    let mut res = Vec::new();
    for item in spider_res.into_iter() {
        let tmp = VirtualLabGrade {
            lab_name: item.LabName,
            score: if item.LabScore.is_empty() {
                None
            } else {
                Some(item.LabScore)
            },
        };
        res.push(tmp);
    }
    // 可能会有重复的，需要去重
    res.sort_by(|a, b| a.lab_name.cmp(&b.lab_name));
    res.dedup_by(|a, b| a.lab_name == b.lab_name);
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lab::test::{TEST_COURSE_ID, TEST_SEMESTER_ID, get_lab_token};

    #[tokio::test]
    #[ignore]
    async fn test_get_lab_grade() {
        let lab_token = get_lab_token().await.unwrap();
        let grade = get_lab_grade(&lab_token, TEST_COURSE_ID, TEST_SEMESTER_ID)
            .await
            .unwrap();
        println!("{:#?}", grade);
    }
}
