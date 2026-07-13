mod fetch;
mod parse;

use crate::{pt::login::PtToken, utils::obs};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 校园卡信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardInfo {
    /// 校园卡账号
    pub id: u32,
    /// 校园卡余额
    // TODO 解析成整数
    pub balance: f64,
}

/// 校园卡消费历史类型
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardHistoryType {
    /// 充值
    Recharge,
    /// 消费
    Consumption,
}

/// 校园卡消费历史详情
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardHistory {
    /// 总交易金额
    ///
    /// 如果是充值金额则是正数，如果是消费金额则是负数
    pub total: f64,
    /// 交易数量
    pub count: u32,
    /// 交易项列表
    pub items: Vec<CardHistoryItem>,
}

/// 校园卡消费历史的交易项
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardHistoryItem {
    /// 交易时间
    pub date_time: NaiveDateTime,
    /// 记账时间
    pub journal_time: NaiveDateTime,
    /// 交易状态，比如 `正常`
    pub status: String,
    /// 交易 id
    pub id: u32,
    /// 交易后余额
    pub now_balance: f64,
    /// 交易金额
    ///
    /// 如果是充值金额则是正数，如果是消费金额则是负数
    pub amount: f64,
    /// 交易地点
    pub location: Option<String>,
    /// 交易名称
    pub name: String,
}

/// 获取校园卡信息
///
/// # Arguments
///
/// - `pt_token`: 个人门户令牌，可以通过 [PtToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 校园卡信息
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(pt_token), fields(subsystem = "pt"), err)
)]
pub async fn get_card_info(pt_token: &PtToken) -> Result<CardInfo, crate::Error<Infallible>> {
    let json_str = fetch::card_info(pt_token).await?;
    parse::card_info(&json_str)
}

/// 获取校园卡消费历史
///
/// # Arguments
///
/// - `pt_token`: 个人门户令牌，可以通过 [PtToken::acquire_by_cas_login] 获取
/// - `year`: 年份
/// - `month`: 月份
/// - `history_type`: 查询充值记录还是消费记录
///
/// # Returns
///
/// 校园卡消费历史信息
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(pt_token), fields(subsystem = "pt"), err)
)]
pub async fn get_card_history(
    pt_token: &PtToken,
    year: u16,
    month: u8,
    history_type: CardHistoryType,
) -> Result<CardHistory, crate::Error<Infallible>> {
    let trancode = match history_type {
        CardHistoryType::Consumption => "15",
        CardHistoryType::Recharge => "16",
    };
    let csrf_token = {
        let _s = obs::debug_span!("fetch_csrf");
        let json_str = fetch::csrf_token(pt_token).await?;
        parse::csrf_token(&json_str)?
    };
    let history = {
        let _s = obs::debug_span!("fetch_history");
        let json_str = fetch::card_history(pt_token, &csrf_token, year, month, trancode).await?;
        parse::card_history(&json_str)?
    };
    obs::debug!(count = history.count, "query_success");
    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pt::test::get_pt_token,
        test::{TEST_MONTH, TEST_YEAR, TestResult},
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_card_info() -> TestResult<()> {
        let token = get_pt_token().await?;
        let res = get_card_info(&token).await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_card_history() -> TestResult<()> {
        let token = get_pt_token().await?;
        let card_history = get_card_history(
            &token,
            *TEST_YEAR,
            *TEST_MONTH,
            CardHistoryType::Consumption,
        )
        .await?;
        println!("{:#?}", card_history);
        Ok(())
    }
}
