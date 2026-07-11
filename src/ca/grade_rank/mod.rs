mod fetch;
mod parse;

use crate::ca::login::CaToken;
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
    let file_name =
        fetch::preview_file(ca_token, fetch::UNDERGRADUATE_MAJOR_ALL_TEMPLATE_ID).await?;
    let file_name = parse::preview_file_name(&file_name)?;
    let pdf_bytes = fetch::file(ca_token, &file_name).await?;
    parse::rank(pdf_bytes)
}

#[cfg(test)]
mod tests {
    use super::get_grade_rank;
    use crate::{ca::test::get_ca_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_grade_rank() -> TestResult<()> {
        let ca_token = get_ca_token().await?;
        let grade_rank = get_grade_rank(&ca_token).await?;
        println!("{:#?}", grade_rank);
        Ok(())
    }
}
