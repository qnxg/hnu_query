mod raw;

use crate::{
    error::MapParseErr,
    pt::{
        card::raw::{raw_card_history_data, raw_card_info_data},
        login::PtToken,
    },
};
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
pub async fn get_card_info(pt_token: &PtToken) -> Result<CardInfo, crate::Error<Infallible>> {
    let res = raw_card_info_data(pt_token).await?;
    Ok(CardInfo {
        id: res.account,
        balance: res.balance.parse::<f64>().parse_err(&res.balance)? / 100.0,
    })
}

/// 获取校园卡消费历史
///
/// # Parameters
///
/// - `pt_token`: 个人门户令牌，可以通过 [PtToken::acquire_by_cas_login] 获取
/// - `year`: 年份
/// - `month`: 月份
/// - `history_type`: 查询充值记录还是消费记录
///
/// # Returns
///
/// 校园卡消费历史信息
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
    let raw_data = raw_card_history_data(pt_token, year, month, trancode).await?;
    let raw_items = raw_data.webTrjnDTO.unwrap_or_default();
    let mut items = Vec::with_capacity(raw_items.len());
    for item in raw_items {
        let date_time = NaiveDateTime::parse_from_str(&item.effectdate, "%Y/%m/%d %H:%M:%S")
            .parse_err_with_reason(&item.effectdate, "date_time")?;
        let journal_time = NaiveDateTime::parse_from_str(&item.jndatetime, "%Y/%m/%d %H:%M:%S")
            .parse_err_with_reason(&item.jndatetime, "journal_time")?;
        let now_balance = item
            .nowAmt
            .parse::<f64>()
            .parse_err_with_reason(&item.nowAmt, "now_balance")?;
        let amount = item
            .fTranAmt
            .parse::<f64>()
            .parse_err_with_reason(&item.fTranAmt, "amount")?;
        items.push(CardHistoryItem {
            date_time,
            journal_time,
            status: item.jourName,
            id: item.usedcardnum,
            now_balance,
            amount,
            location: item.sysname1.map(|s| s.trim().to_string()),
            name: item.tranname,
        });
    }
    let res = CardHistory {
        total: raw_data.amt / 100.0,
        count: raw_data.count as u32,
        items,
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pt::test::get_pt_token,
        test::{TEST_MONTH, TEST_YEAR},
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_card_info() {
        let token = get_pt_token().await.unwrap();
        let res = get_card_info(&token).await.unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_card_history() {
        let token = get_pt_token().await.unwrap();
        let card_history = get_card_history(
            &token,
            *TEST_YEAR,
            *TEST_MONTH,
            CardHistoryType::Consumption,
        )
        .await
        .unwrap();
        println!("{:#?}", card_history);
    }
}
