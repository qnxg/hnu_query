mod fetch;
mod parse;

use crate::hdjw::{error::TokenExpired, login::HdjwToken};
use serde::{Deserialize, Serialize};

/// 排名具体信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RankDetail {
    /// 算数平均成绩
    pub arithmetic: String,
    /// 算数平均成绩排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub arithmetic_rank: String,
    /// 加权平均成绩
    pub weighted: String,
    /// 加权平均成绩排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub weighted_rank: String,
    /// 平均学分绩点
    pub gpa: String,
    /// 平均学分绩点排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub gpa_rank: String,
}

/// 排名
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rank {
    /// 全部课程的排名信息
    ///
    /// 为 `None` 说明暂无数据
    pub all: Option<RankDetail>,
    /// 必修课程的排名信息
    ///
    /// 为 `None` 说明暂无数据
    pub must: Option<RankDetail>,
    /// 核心课程的排名信息
    ///
    /// 为 `None` 说明暂无数据
    pub core: Option<RankDetail>,
}

/// 方案类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Range {
    /// 主修
    Major,
    /// 辅修
    Minor,
}

impl Range {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Range::Major => "0",
            Range::Minor => "1",
        }
    }
}

/// 数据来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSource {
    /// 成绩总库
    Total,
    /// 执行方案
    Execution,
}

impl DataSource {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            DataSource::Total => "1",
            DataSource::Execution => "2",
        }
    }
}

/// 显示方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Display {
    /// 最大成绩
    Max,
    /// 初修成绩
    Initial,
}

impl Display {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Display::Max => "0",
            Display::Initial => "1",
        }
    }
}

/// 获取排名
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌，可以通过 [HdjwToken::acquire_by_cas_login] 获取
/// - `selection`: 学年学期，应提供一个二元组的切片，切片中每个二元组的格式为 `(学年, 学期)`，
///   如果为空，则表示获取所有学年学期的排名
/// - `range`: 主修还是辅修
/// - `data_source`: 数据来源
/// - `display`: 取最大成绩还是初修成绩
///
/// # Returns
///
/// 返回一个排名结果，如果没有获取到任何数据，则返回 `None`
///
/// # Errors
///
/// 如果提供的 `hdjw_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [HdjwToken]
pub async fn get_rank(
    hdjw_token: &HdjwToken,
    selection: &[(u16, u8)],
    range: Range,
    data_source: DataSource,
    display: Display,
) -> Result<Rank, crate::Error<TokenExpired>> {
    let selection = selection
        .iter()
        .map(|(xn, xq)| format!("{}-{}-{}", xn, xn + 1, xq))
        .collect::<Vec<_>>()
        .join(",");
    let json_str = fetch::rank(
        hdjw_token,
        &selection,
        range.as_str(),
        data_source.as_str(),
        display.as_str(),
    )
    .await?;
    parse::rank(&json_str)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        hdjw::test::get_hdjw_token,
        test::{TEST_XN, TEST_XQ, TestResult},
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_rank() -> TestResult<()> {
        let hdjw_token = get_hdjw_token().await?;
        let selection = vec![(*TEST_XN, *TEST_XQ)];
        let rank = get_rank(
            &hdjw_token,
            &selection,
            Range::Major,
            DataSource::Total,
            Display::Max,
        )
        .await?;
        println!("{:#?}", rank);
        Ok(())
    }
}
