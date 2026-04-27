mod raw;

use crate::{
    error::parse_err,
    hdjw::{
        error::TokenExpired,
        grade::raw::{raw_grade_data, raw_grade_detail_data},
        login::HdjwToken,
    },
};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    pub gpa: f32,
    /// 该门课程获得的分数
    pub score: u8,
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
    let raw_data = raw_grade_data(hdjw_token, xn, xq).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let grade = Grade {
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
        };
        res.push(grade);
    }
    Ok(res)
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
    let raw_data = raw_grade_detail_data(hdjw_token, jx0404id).await?;
    let regex =
        RegexBuilder::new(r"let\sarr\s=\s(.*);.*window.initQzTable\(\{.*cols:\s\[(.*)\].*\}\);")
            .dot_matches_new_line(true)
            .build()
            .expect("构建正则表达式失败");
    let caps = regex
        .captures(&raw_data)
        .ok_or(parse_err(&raw_data))?
        .iter()
        .map(|c| {
            c.map(|v| v.as_str().to_string())
                .ok_or(parse_err(&raw_data))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [_, data, map] = caps.try_into().map_err(|_| parse_err(&raw_data))?;
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
                        .ok_or(parse_err(&raw_data))
                        .map(|ok_value| (key, ok_value))
                })
                .collect::<Result<HashMap<_, _>, _>>()
        })
        .ok_or(parse_err(&raw_data))??;
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
                    item.get("title")
                        .and_then(|f| f.as_str())
                        .and_then(|value| key.map(|key| (key, value)))
                        .ok_or(parse_err(&raw_data))
                })
                .collect::<Result<HashMap<_, _>, _>>()
        })
        .ok_or(parse_err(&raw_data))??;
    let res = data
        .iter()
        .filter(|(k, _)| k.ends_with("bl"))
        .map(|(k, v)| {
            let score = data
                .get(&k.trim_end_matches("bl").to_string())
                .ok_or(parse_err(&raw_data))?;
            let name = map
                .get(k.trim_end_matches("bl"))
                .ok_or(parse_err(&raw_data))?;
            let percentage = v;
            Ok::<_, crate::Error<TokenExpired>>(GradeDetailItem {
                score: score.to_string(),
                name: name.to_string(),
                percentage: percentage.to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|item| item.percentage != "0%")
        .collect::<Vec<_>>();
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        hdjw::test::{TEST_HDJW_JX0404ID, get_hdjw_token},
        test::{TEST_XN, TEST_XQ},
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_grade() {
        let hdjw_token = get_hdjw_token().await.unwrap();
        let grade = get_grade(&hdjw_token, *TEST_XN, *TEST_XQ).await.unwrap();
        println!("{:#?}", grade);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_grade_detail() {
        let hdjw_token = get_hdjw_token().await.unwrap();
        let grade_detail = get_grade_detail(&hdjw_token, TEST_HDJW_JX0404ID)
            .await
            .unwrap();
        println!("{:#?}", grade_detail);
    }
}
