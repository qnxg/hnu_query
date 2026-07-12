use crate::{
    error::{MapParseErr, parse_err},
    lab::grade::{LabGrade, LabGradeDetailItem, VirtualLabGrade},
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, convert::Infallible};

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawLabGrade {
    /// 出勤情况
    AttendanceName: String,
    /// 实验名称
    LabName: String,
    /// 实验成绩，没有成绩的话是空字符串
    LabScore: String,
    /// 实验id
    LabID: String,
    /// 上课地点，这个字段只是用来判断是否为虚拟实验的
    ClassRoom: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawLabGradeDetail {
    /// 对应的成绩结构id
    LabScoreStructureID: i32,
    /// 对应的实验id
    LabID: i32,
    /// 分数
    LabStructureScore: Option<f64>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawLabGradeStructure {
    /// 成绩结构id
    LabScoreStructureID: i32,
    /// 成绩结构名称
    LabScoreStructureName: String,
}

fn parse_lab_grade(json_str: &str) -> Result<Vec<RawLabGrade>, crate::Error<Infallible>> {
    serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("rows")
        .map(|v| serde_json::from_value::<Vec<RawLabGrade>>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))
}

fn parse_lab_grade_detail(
    json_str: &str,
) -> Result<Vec<RawLabGradeDetail>, crate::Error<Infallible>> {
    serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("Data")
        .and_then(|v| v.get("Lablist"))
        .map(|v| serde_json::from_value::<Vec<RawLabGradeDetail>>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))
}

fn parse_lab_grade_structure(
    json_str: &str,
) -> Result<Vec<RawLabGradeStructure>, crate::Error<Infallible>> {
    serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("Data")
        .map(|v| serde_json::from_value::<Vec<RawLabGradeStructure>>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))
}

/// # Arguments
///
/// - `lab_grade_str`: 由 [`super::fetch::lab_grade`] 返回的数据
/// - `lab_grade_detail_str`: 由 [`super::fetch::lab_grade_detail`] 返回的数据
/// - `lab_grade_structure_str`: 由 [`super::fetch::lab_grade_structure`] 返回的数据
pub fn lab_grade(
    lab_grade_str: &str,
    lab_grade_detail_str: &str,
    lab_grade_structure_str: &str,
) -> Result<Vec<LabGrade>, crate::Error<Infallible>> {
    let lab_score = parse_lab_grade(lab_grade_str)?;
    let lab_score_detail = parse_lab_grade_detail(lab_grade_detail_str)?;
    let lab_score_structure = parse_lab_grade_structure(lab_grade_structure_str)?;
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

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawVirtualLabGrade {
    /// 实验名称
    LabName: String,
    /// 实验成绩，没有成绩的话是空字符串
    LabScore: String,
}

/// `json_str` 为 [super::fetch::virtual_lab_grade] 的返回数据
pub fn virtual_lab_grade(json_str: &str) -> Result<Vec<VirtualLabGrade>, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("rows")
        .map(|v| serde_json::from_value::<Vec<RawVirtualLabGrade>>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;
    let mut res = Vec::new();
    for item in raw_data.into_iter() {
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
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_lab_grade() -> TestResult<()> {
        let raw = parse_lab_grade(include_str!("test_data/GetStudentLabScore.json"))?;
        assert_eq!(raw.len(), 3);
        let first = &raw[0];
        assert_eq!(first.LabName, "绪论");
        assert_eq!(first.LabScore, "84.4");
        assert_eq!(first.LabID, "415");
        assert_eq!(first.AttendanceName, "正常");
        assert_eq!(first.ClassRoom, "研究生楼");
        Ok(())
    }

    #[test]
    fn test_parse_lab_grade_detail() -> TestResult<()> {
        let raw = parse_lab_grade_detail(include_str!("test_data/ShowScore.json"))?;
        assert_eq!(raw.len(), 8);
        let first = &raw[0];
        assert_eq!(first.LabScoreStructureID, 58);
        assert_eq!(first.LabID, 415);
        assert_eq!(first.LabStructureScore, Some(100.0));
        Ok(())
    }

    #[test]
    fn test_parse_lab_grade_structure() -> TestResult<()> {
        let raw = parse_lab_grade_structure(include_str!("test_data/GetLabScoreStructure.json"))?;
        assert_eq!(raw.len(), 4);
        let first = &raw[0];
        assert_eq!(first.LabScoreStructureID, 58);
        assert_eq!(first.LabScoreStructureName, "预习");
        Ok(())
    }

    #[test]
    fn test_lab_grade() -> TestResult<()> {
        let grades = lab_grade(
            include_str!("test_data/GetStudentLabScore.json"),
            include_str!("test_data/ShowScore.json"),
            include_str!("test_data/GetLabScoreStructure.json"),
        )?;
        assert_eq!(grades.len(), 3);
        // 绪论
        let lab = &grades[0];
        assert_eq!(lab.lab_name, "绪论");
        assert_eq!(lab.score, "84.4");
        assert_eq!(lab.attendance, Some("正常".to_string()));
        assert_eq!(lab.details.len(), 4);
        assert_eq!(lab.details[0].name, "预习");
        assert_eq!(lab.details[0].score, Some(100.0));
        assert_eq!(lab.details[1].name, "纪律");
        assert_eq!(lab.details[1].score, Some(100.0));
        assert_eq!(lab.details[2].name, "操作");
        assert_eq!(lab.details[2].score, Some(100.0));
        assert_eq!(lab.details[3].name, "报告");
        assert_eq!(lab.details[3].score, Some(44.0));
        // 物体密度测量
        let lab = &grades[1];
        assert_eq!(lab.lab_name, "物体密度测量");
        assert_eq!(lab.score, "87.5");
        assert_eq!(lab.attendance, Some("正常".to_string()));
        assert_eq!(lab.details.len(), 4);
        assert_eq!(lab.details[0].name, "预习");
        assert_eq!(lab.details[0].score, Some(75.0));
        assert_eq!(lab.details[1].name, "纪律");
        assert_eq!(lab.details[1].score, Some(100.0));
        assert_eq!(lab.details[2].name, "操作");
        assert_eq!(lab.details[2].score, Some(80.0));
        assert_eq!(lab.details[3].name, "报告");
        assert_eq!(lab.details[3].score, Some(75.0));
        // 冷却法测量金属的比热容（无成绩组成详情）
        let lab = &grades[2];
        assert_eq!(lab.lab_name, "冷却法测量金属的比热容");
        assert_eq!(lab.score, "84");
        assert_eq!(lab.attendance, Some("正常".to_string()));
        assert_eq!(lab.details.len(), 0);
        Ok(())
    }
}
