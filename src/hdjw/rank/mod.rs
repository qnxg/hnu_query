mod raw;

use crate::hdjw::{error::TokenExpired, login::HdjwToken};
use serde::{Deserialize, Serialize};

/// 参与计算成绩排名的课程范围
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankRange {
    /// 通识必修
    GeneralRequired,
    /// 通识选修
    GeneralElective,
    /// 专业选修
    MajorElective,
    /// 专业基础
    MajorBasic,
    /// 专业核心
    MajorCore,
    /// 学类核心
    ClusterCore,
    /// 学门核心
    GatewayCore,
    /// 实践环节
    Practice,
    /// 创新创业
    Innovation,
    /// 国际化
    International,
    /// 马克思主义经典
    ///
    /// WARNING：这个类型并没有在教务系统中明确列出，而是隐藏在全部课程中。
    /// 我们推测是教务系统的这部分没有对湖大做本地适配，实际上这个类型不存在。
    MarxismClassic,
    /// 科学与艺术经典
    ///
    /// WARNING：这个类型并没有在教务系统中明确列出，而是隐藏在全部课程中。
    /// 我们推测是教务系统的这部分没有对湖大做本地适配，实际上这个类型不存在。
    ScienceAndArtClassic,
    /// 西方经典
    ///
    /// WARNING：这个类型并没有在教务系统中明确列出，而是隐藏在全部课程中。
    /// 我们推测是教务系统的这部分没有对湖大做本地适配，实际上这个类型不存在。
    WesternClassic,
    /// 中国经典
    ///
    /// WARNING：这个类型并没有在教务系统中明确列出，而是隐藏在全部课程中。
    /// 我们推测是教务系统的这部分没有对湖大做本地适配，实际上这个类型不存在。
    ChineseClassic,
    /// 其他
    ///
    /// WARNING：这个类型并没有在教务系统中明确列出，而是隐藏在全部课程中。
    /// 我们推测是教务系统的这部分没有对湖大做本地适配，实际上这个类型不存在。
    Other,
    /// 未知字段，这个字段在教务系统中对应的编号为 `19`。
    ///
    /// WARNING：这个类型并没有在教务系统中明确列出，而是隐藏在全部课程中。
    /// 我们推测是教务系统的这部分没有对湖大做本地适配，实际上这个类型不存在。
    Unknown19,
    /// 未知字段，这个字段在教务系统中对应的编号为 `20`。
    ///
    /// WARNING：这个类型并没有在教务系统中明确列出，而是隐藏在全部课程中。
    /// 我们推测是教务系统的这部分没有对湖大做本地适配，实际上这个类型不存在。
    Unknown20,
}

impl RankRange {
    /// 将 [RankRange] 转换为教务系统对应的字符串
    pub(crate) fn to_str(self) -> &'static str {
        match self {
            RankRange::MajorCore => "16",
            RankRange::Practice => "10",
            RankRange::GeneralElective => "15",
            RankRange::GeneralRequired => "11",
            RankRange::ClusterCore => "12",
            RankRange::GatewayCore => "08",
            RankRange::MajorBasic => "03",
            RankRange::Innovation => "17",
            RankRange::International => "88",
            RankRange::MajorElective => "05",
            RankRange::MarxismClassic => "07",
            RankRange::ScienceAndArtClassic => "09",
            RankRange::WesternClassic => "13",
            RankRange::ChineseClassic => "14",
            RankRange::Other => "18",
            RankRange::Unknown19 => "19",
            RankRange::Unknown20 => "20",
        }
    }

    /// 2020 版核心课方案
    ///
    /// 如下课程类型归类为 2020 版核心课方案：
    ///
    /// - 专业核心
    /// - 学类核心
    /// - 学门核心
    ///
    /// # Returns
    ///
    /// 返回一个包含所有2020版核心课程方案类型的列表
    pub fn core_v2020_course() -> Vec<Self> {
        vec![
            RankRange::MajorCore,
            RankRange::ClusterCore,
            RankRange::GatewayCore,
        ]
    }

    /// 2024 版核心课方案
    ///
    /// 如下课程类型归类为 2024 版核心课方案：
    ///
    /// - 专业基础
    /// - 专业核心
    ///
    /// # Returns
    ///
    /// 返回一个包含所有2024版核心课程方案类型的列表
    pub fn core_v2024_course() -> Vec<Self> {
        vec![RankRange::MajorBasic, RankRange::MajorCore]
    }

    /// 全部课程
    ///
    /// # Returns
    ///
    /// 返回一个包含所有课程类型的列表
    pub fn all_cousrse() -> Vec<Self> {
        vec![
            RankRange::GeneralRequired,
            RankRange::GeneralElective,
            RankRange::MajorElective,
            RankRange::MajorBasic,
            RankRange::MajorCore,
            RankRange::ClusterCore,
            RankRange::GatewayCore,
            RankRange::Practice,
            RankRange::Innovation,
            RankRange::International,
            RankRange::MarxismClassic,
            RankRange::ScienceAndArtClassic,
            RankRange::WesternClassic,
            RankRange::ChineseClassic,
            RankRange::Other,
            RankRange::Unknown19,
            RankRange::Unknown20,
        ]
    }

    /// 必修课程
    ///
    /// 如下课程类型归类为必修课程：
    ///
    /// - 通识必修
    /// - 专业基础
    /// - 专业核心
    /// - 学类核心
    /// - 学门核心
    /// - 实践环节
    ///
    /// # Returns
    ///
    /// 返回一个包含所有必修课程类型的列表
    ///
    /// # Notes
    ///
    /// 教务系统中并没有明确给出哪些课程类型属于必修课程，
    /// 这里给出的列表只是我们根据经验得出的，仅供参考。
    pub fn must_course() -> Vec<Self> {
        vec![
            RankRange::GeneralRequired,
            RankRange::MajorBasic,
            RankRange::MajorCore,
            RankRange::ClusterCore,
            RankRange::GatewayCore,
            RankRange::Practice,
        ]
    }
}

/// 排名方式
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankMethod {
    /// 算术平均
    ArithmeticAvg,
    /// 加权平均
    WeightedAvg,
    /// 绩点
    Gpa,
}

impl RankMethod {
    /// 将 [RankMethod] 转换为教务系统对应的字符串
    pub(crate) fn to_str(self) -> &'static str {
        match self {
            RankMethod::ArithmeticAvg => "4",
            RankMethod::WeightedAvg => "2",
            RankMethod::Gpa => "3",
        }
    }
}

/// 排名
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rank {
    /// 排名
    ///
    /// 如果为 `None`，则表示没有获取到排名数据
    pub rank: Option<String>,
    /// 成绩分数
    ///
    /// 如果为 `None`，则表示没有获取到成绩分数数据
    pub score: Option<String>,
}

/// 获取排名
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌，可以通过 [HdjwToken::acquire_by_cas_login] 获取
/// - `selection`: 学年学期，应提供一个二元组的切片，切片中每个二元组的格式为 `(学年, 学期)`
/// - `range`: 课程范围
/// - `rank_method`: 排名计算方式
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
    range: &[RankRange],
    rank_method: RankMethod,
) -> Result<Option<Rank>, crate::Error<TokenExpired>> {
    let selection = selection
        .iter()
        .map(|(xn, xq)| format!("{}-{}-{}", xn, xn + 1, xq))
        .collect::<Vec<_>>()
        .join(",");
    let range = range
        .iter()
        .map(|r| r.to_str())
        .collect::<Vec<&str>>()
        .join(",");
    let Some(res) =
        raw::raw_rank_data(hdjw_token, &selection, &range, rank_method.to_str()).await?
    else {
        return Ok(None);
    };
    let score = match rank_method {
        RankMethod::ArithmeticAvg => res.get("avgzcj").and_then(|v| v.as_str()),
        RankMethod::WeightedAvg => res.get("pjxfj").and_then(|v| v.as_str()),
        RankMethod::Gpa => res.get("pjxfjd").and_then(|v| v.as_str()),
    }
    .map(|s| s.to_string());
    let rank = res
        .get("numrow")
        .and_then(|v| {
            v.as_str()
                .map(|s| s.to_string())
                // 理论上这里的排名数据应该是数字，但是被湖大接口的类型搞怕了，字符串也接受
                .or(v.as_i64().map(|i| i.to_string()))
        })
        .map(|s| s.to_string());
    Ok(Some(Rank { rank, score }))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        hdjw::test::get_hdjw_token,
        test::{TEST_XN, TEST_XQ},
    };

    #[tokio::test]
    #[ignore]
    pub async fn test_get_rank() {
        let hdjw_token = get_hdjw_token().await.unwrap();
        let selection = vec![(*TEST_XN, *TEST_XQ)];
        let range = RankRange::core_v2024_course();
        let rank_method = RankMethod::WeightedAvg;
        let rank = get_rank(&hdjw_token, &selection, &range, rank_method)
            .await
            .unwrap();
        println!("{:#?}", rank);
    }
}
