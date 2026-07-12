use super::{Grade, GradeDetailItem};
use crate::{
    error::{MapParseErr, parse_err},
    hdjw::error::TokenExpired,
};
use regex::{Regex, RegexBuilder};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, sync::LazyLock};

/// 教务 `考试成绩 > 课程成绩` 返回数据单项
#[derive(Deserialize, Debug)]
struct RawGradeInfo {
    // 未知字段
    // cj0708id: String,
    // 学年学期信息（暂时不用）
    // xnxqid: String,
    /// 课程代码
    kch: String,
    /// 课程名称
    kc_mc: String,
    // 开课学院（暂时不用）
    // ksdw: String,
    // 似乎和 xnxqid 重复
    // xqmc: String,
    /// 学分
    xf: f32,
    // 总学时（暂时不用）
    // zxs: u32,
    // 考试方式（暂时不用）
    // ksfs: String,
    /// 课程属性（必修/选修等）
    kcsx: Option<String>,
    // 似乎又和 xnxqid 重复
    // xqstr: String,
    /// 总成绩
    zcj: f64,
    // 总成绩字符串形式（暂时不用）
    // zcjstr: String,
    // 未知字段
    // kz: u8,
    /// 课程性质（通识必修/专业核心等）
    kcxzmc: String,
    // 未知字段
    // xs0101id: String,
    /// 用于课程成绩详情查询，部分成绩没有该字段
    jx0404id: Option<String>,
    /// 绩点
    ///
    /// 有的课程没有这个数据
    jd: Option<f32>,
    // 考试性质（暂时不用）
    // pub ksxz: String,
    /// 主修还是辅修
    falb: String,
    /// 成绩标识（缓考/重修等，注意这个标识是挂在为 0 分的那个成绩 item 上）
    cjbs: Option<String>,
}

/// `json_str` 为 [`super::fetch::grade`] 返回的数据
pub fn grade(json_str: &str) -> Result<Vec<Grade>, crate::Error<TokenExpired>> {
    let json = crate::hdjw::parse::hdjw_response(json_str)?;
    let raw_data =
        serde_json::from_value::<Vec<RawGradeInfo>>(json["data"].clone()).parse_err(json_str)?;
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

/// `html` 为 [`super::fetch::grade_detail`] 返回的数据
pub fn grade_detail(html: &str) -> Result<Vec<GradeDetailItem>, crate::Error<TokenExpired>> {
    let json = crate::hdjw::parse::hdjw_response(html)?;
    // 不要直接 to_string()，
    // 否则会把整段 HTML 当作 JSON 字符串再序列化，导致内部引号变成 \"
    let raw_data = json.as_str().ok_or_else(|| parse_err(html))?.to_string();
    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
        RegexBuilder::new(r"let\sarr\s=\s(.*);.*window.initQzTable\(\{.*cols:\s\[(.*)\].*\}\);")
            .dot_matches_new_line(true)
            .build()
            .expect("构建正则表达式失败")
    });
    let caps = REGEX
        .captures(&raw_data)
        .ok_or_else(|| parse_err(&raw_data))?
        .iter()
        .map(|c| {
            c.map(|v| v.as_str().to_string())
                .ok_or_else(|| parse_err(&raw_data))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [_, data, map] = caps.try_into().map_err(|_| parse_err(&raw_data))?;
    let data = serde_json::from_str::<Vec<Value>>(&data).parse_err(&raw_data)?;
    let data = data
        .first()
        .and_then(|v| v.as_object())
        .map(|v| {
            v.iter()
                .map(|(key, value)| {
                    value
                        .as_str()
                        .map(|s| s.to_string())
                        .or(value.as_number().map(|num| num.to_string()))
                        .ok_or_else(|| parse_err(&raw_data))
                        .map(|ok_value| (key, ok_value))
                })
                .collect::<Result<HashMap<_, _>, _>>()
        })
        .ok_or_else(|| parse_err(&raw_data))??;
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
                        .ok_or_else(|| parse_err(&raw_data))
                })
                .collect::<Result<HashMap<_, _>, _>>()
        })
        .ok_or_else(|| parse_err(&raw_data))??;
    let res = data
        .iter()
        .filter(|(k, _)| k.ends_with("bl"))
        .map(|(k, v)| {
            let score = data
                .get(&k.trim_end_matches("bl").to_string())
                .ok_or_else(|| parse_err(&raw_data))?;
            let name = map
                .get(k.trim_end_matches("bl"))
                .ok_or_else(|| parse_err(&raw_data))?;
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
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_grade() -> TestResult<()> {
        let grades = grade(include_str!("test_data/cjcx_list.json"))?;

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
