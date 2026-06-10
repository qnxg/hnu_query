use crate::{
    error::parse_err,
    hdjw::{error::TokenExpired, grade::raw::RawGradeInfo},
};
use regex::RegexBuilder;
use serde_json::Value;
use std::collections::HashMap;

use super::{Grade, GradeDetailItem};

pub fn grade(raw_data: Vec<RawGradeInfo>) -> Result<Vec<Grade>, crate::Error<TokenExpired>> {
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        res.push(Grade {
            course_id: item.kch,
            course_name: item.kc_mc,
            credit: item.xf,
            course_type1: item.kcsx,
            course_type2: item.kcxzmc,
            gpa: item.jd,
            score: item.zcj,
            grade_tag: item.cjbs,
            grade_type: item.falb,
            jx0404id: item.jx0404id,
        });
    }
    Ok(res)
}

/// # Parameters
///
/// - `raw_data`: 由 [super::raw::get_pscj_list] 返回的原始 HTML 文本数据
pub fn grade_detail(raw_data: &str) -> Result<Vec<GradeDetailItem>, crate::Error<TokenExpired>> {
    let regex =
        RegexBuilder::new(r"let\sarr\s=\s(.*);.*window.initQzTable\(\{.*cols:\s\[(.*)\].*\}\);")
            .dot_matches_new_line(true)
            .build()
            .unwrap_or_else(|e| panic!("构建正则表达式失败: {:?}", e));
    let caps = regex
        .captures(raw_data)
        .ok_or(parse_err(raw_data))?
        .iter()
        .map(|c| c.map(|v| v.as_str().to_string()).ok_or(parse_err(raw_data)))
        .collect::<Result<Vec<_>, _>>()?;
    let [_, data, map] = caps.try_into().map_err(|_| parse_err(raw_data))?;
    let data = serde_json::from_str::<Vec<Value>>(&data).ok();
    let data = data
        .as_ref()
        .and_then(|v| v.first())
        .and_then(|v| v.as_object())
        .map(|v| {
            v.iter()
                .map(|(key, value)| {
                    value
                        .as_str()
                        .map(|s| s.to_string())
                        .or(value.as_number().map(|num| num.to_string()))
                        .ok_or(parse_err(raw_data))
                        .map(|ok_value| (key, ok_value))
                })
                .collect::<Result<HashMap<_, _>, _>>()
        })
        .ok_or(parse_err(raw_data))??;
    // map 是 js obj 格式，不是标准 json，我们需要进行一些处理
    let map = map
        .replace("//表头", "")
        .replace("'", "\"")
        .replace("field", "\"field\"")
        .replace("title", "\"title\"")
        .replace("type", "\"type\"");
    let map = serde_json::from_str::<Value>(map.as_str()).ok();
    let map = map
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|v| {
            v.iter()
                .filter(|item| item.get("field").and_then(|f| f.as_str()).is_some())
                .map(|item| {
                    let key = item.get("field").and_then(|f| f.as_str());
                    key.zip(item.get("title").and_then(|f| f.as_str()))
                        .ok_or(parse_err(raw_data))
                })
                .collect::<Result<HashMap<_, _>, _>>()
        })
        .ok_or(parse_err(raw_data))??;
    let res = data
        .iter()
        .filter(|(k, _)| k.ends_with("bl"))
        .map(|(k, v)| {
            let score = data
                .get(&k.trim_end_matches("bl").to_string())
                .ok_or(parse_err(raw_data))?;
            let name = map
                .get(k.trim_end_matches("bl"))
                .ok_or(parse_err(raw_data))?;
            let percentage = v;
            Ok::<_, crate::Error<TokenExpired>>(GradeDetailItem {
                score: score.clone(),
                name: name.to_string(),
                percentage: percentage.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|item| item.percentage != "0%")
        .collect::<Vec<_>>();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_grade() -> TestResult<()> {
        let raw_data: Vec<RawGradeInfo> =
            serde_json::from_str(include_str!("test_data/cjcx_list.json"))?;

        let grades = grade(raw_data)?;

        assert_eq!(grades.len(), 4);

        let first_item = &grades[0];
        assert_eq!(first_item.course_id, "TB001XK24B");
        assert_eq!(first_item.course_name, "计算与人工智能概论B");
        assert_eq!(first_item.credit, 4.0);
        assert_eq!(first_item.course_type1, Some("必修".to_string()));
        assert_eq!(first_item.course_type2, "通识必修");
        assert_eq!(first_item.gpa, Some(4.0));
        assert_eq!(first_item.score, 92.0);
        assert_eq!(first_item.grade_tag, None);
        assert_eq!(first_item.grade_type, "主修");
        assert_eq!(first_item.jx0404id, Some("TB001XK24B-155".to_string()));

        Ok(())
    }

    #[test]
    fn test_grade_detail() -> TestResult<()> {
        let raw_data = include_str!("test_data/pscj_list.html");

        let mut detail = grade_detail(raw_data)?;
        // 排序仅用于后续数组的 index 能正确对应
        detail.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(detail.len(), 4);

        assert_eq!(detail[0].name, "平时成绩1");
        assert_eq!(detail[0].score, "99.83");
        assert_eq!(detail[0].percentage, "30%");

        assert_eq!(detail[1].name, "平时成绩2");
        assert_eq!(detail[1].score, "67.5");
        assert_eq!(detail[1].percentage, "20%");

        assert_eq!(detail[2].name, "期中成绩");
        assert_eq!(detail[2].score, "98");
        assert_eq!(detail[2].percentage, "10%");

        assert_eq!(detail[3].name, "期末成绩");
        assert_eq!(detail[3].score, "98");
        assert_eq!(detail[3].percentage, "40%");

        Ok(())
    }
}
