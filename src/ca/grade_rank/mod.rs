mod raw;

use crate::{
    ca::{
        grade_rank::raw::{UNDERGRADUATE_MAJOR_ALL_TEMPLATE_ID, raw_certification_data},
        login::CaToken,
    },
    error::parse_err,
};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 可信电子凭证中的排名
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rank {
    /// 全部课程的平均学分绩点
    pub all_gpa: String,
    /// 全部课程的平均学分绩点排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub all_gpa_rank: String,
    /// 全部课程的加权平均分
    pub all_weighted: String,
    /// 全部课程的加权平均分排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub all_weighted_rank: String,
    /// 全部课程的算术平均分
    pub all_arithmetic: String,
    /// 全部课程的算术平均分排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub all_arithmetic_rank: String,
    /// 必修课的平均学分绩点
    pub must_gpa: String,
    /// 必修课的加权平均分
    pub must_weighted: String,
    /// 必修课的算术平均分
    pub must_arithmetic: String,
    /// 核心课程的平均学分绩点排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub core_gpa_rank: String,
    /// 核心课程的加权平均分排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub core_weighted_rank: String,
    /// 核心课程的算术平均分排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub core_arithmetic_rank: String,
}

/// 获取本科生可信电子凭证中的成绩排名
///
/// 仅计算主修课，辅修课不计算在内
///
/// # Arguments
///
/// - `ca_token`: 可信电子凭证的令牌，可以通过 [CaToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 可信电子凭证中的成绩排名信息
pub async fn get_grade_rank(ca_token: &CaToken) -> Result<Rank, crate::Error<Infallible>> {
    let raw_data = raw_certification_data(ca_token, UNDERGRADUATE_MAJOR_ALL_TEMPLATE_ID).await?;
    let regex = RegexBuilder::new(r"平均学分绩点排名 ([0-9/]+).*平均学分绩点 ([0-9.]+).*核心课程平均学分绩点排名 ([0-9/]+).*必修课平均学分绩点 ([0-9.]+).*课程算术平均成绩排名 ([0-9/]+).*算术平均分 ([0-9.]+).*核心课程算术平均成绩排名 ([0-9/]+).*必修课算术平均分 ([0-9.]+).*学分加权平均成绩排名 ([0-9/]+).*加权平均分 ([0-9.]+).*核心课程学分加权平均成绩排名 ([0-9/]+).*必修课加权平均分 ([0-9.]+)")
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
    // 12 个捕获组，caps[0] 是完整匹配，共 13 个
    let [
        _,
        all_gpa_rank,
        all_gpa,
        core_gpa_rank,
        must_gpa,
        all_arithmetic_rank,
        all_arithmetic,
        core_arithmetic_rank,
        must_arithmetic,
        all_weighted_rank,
        all_weighted,
        core_weighted_rank,
        must_weighted,
    ] = caps.try_into().map_err(|_| parse_err(&raw_data))?;
    let res = Rank {
        all_gpa,
        all_gpa_rank,
        all_weighted,
        all_weighted_rank,
        all_arithmetic,
        all_arithmetic_rank,
        must_gpa,
        must_weighted,
        must_arithmetic,
        core_gpa_rank,
        core_arithmetic_rank,
        core_weighted_rank,
    };
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::get_grade_rank;
    use crate::ca::test::get_ca_token;

    #[tokio::test]
    #[ignore]
    pub async fn test_get_grade_rank() {
        let ca_token = get_ca_token().await.unwrap();
        let grade_rank = get_grade_rank(&ca_token).await.unwrap();
        println!("{:#?}", grade_rank);
    }
}
