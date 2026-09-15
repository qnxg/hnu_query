//! 校园卡账户与交易明细查询

mod fetch;
mod parse;

use crate::{
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::obs::{fetch_time, parse_time},
};
use chrono::{NaiveDate, NaiveDateTime};
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};

/// 校园卡账户信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardInfo {
    /// 校园卡账号
    pub account: String,
    /// 校园卡余额, 单位为人民币元
    pub balance: f64,
}

/// 一条校园卡交易记录
///
/// **交易金额单位为人民币分**
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionRecord {
    /// 商户名称
    pub merchant_name: String,
    /// 商户账号
    pub merchant_account: u64,
    /// 交易时间
    pub pay_time: NaiveDateTime,
    /// 交易金额, 正值为收入, 负值为支出, 单位为分
    pub transaction_amount: i64,
    /// 交易类型
    pub transaction_type: TransactionType,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// 交易类型, 枚举可能不完全
pub enum TransactionType {
    /// 消费
    Consumption = 15,
    /// 充值
    Recharge = 16,
    /// 领取补助 (电子账户 -> 校园卡), **注意: 不一定是正值**
    /// 计算总收入/支出金额时需要排除掉这个类型
    // 这个交易类型语义不明确, 可能是正值, 也可能是负值
    MoneyTransfer = 22,
    /// 补助
    Benefit = 17,
    /// 电子账户开户
    EAccountOpening = 6,
    /// 持卡人开户
    HolderAccountOpening = 1,
    /// 其他
    Other(u32),
}

impl TransactionType {
    /// 获取对应的交易类型代码
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(self), fields(subsystem = "iportal"))
    )]
    pub fn get_type_code(&self) -> u32 {
        match self {
            TransactionType::Consumption => 15,
            TransactionType::Recharge => 16,
            TransactionType::MoneyTransfer => 22,
            TransactionType::Benefit => 17,
            TransactionType::EAccountOpening => 6,
            TransactionType::HolderAccountOpening => 1,
            TransactionType::Other(code) => *code,
        }
    }
}

/// 分页的校园卡交易明细
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardTransactionDetail {
    /// 是否还有下一页
    ///
    /// 当前页之后是否还有记录
    pub has_next_page: bool,
    /// 符合查询条件的记录总数
    pub total_count: u32,
    /// 当前页的交易记录
    pub records: Vec<TransactionRecord>,
}

/// 获取校园卡信息
///
/// # Arguments
///
/// - `token`: iportal 令牌, 可以通过
///   [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login)
///   获取
///
/// # Returns
///
/// 返回校园卡信息
///
/// # Errors
///
/// `token` 失效时返回 [`IPortalTokenExpired`]
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_card_info(
    token: &IPortalToken,
) -> Result<CardInfo, crate::Error<IPortalTokenExpired>> {
    let json_str = fetch_time!(fetch::balance(token).await)?;
    let info = parse_time!(parse::card_info(&json_str))?;
    Ok(info)
}

/// 分页获取指定日期范围内的校园卡交易明细
///
/// # Arguments
///
/// - `token`: iportal 令牌, 可以通过
///   [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login)
///   获取
/// - `start`: 查询开始日期
/// - `end`: 查询结束日期
/// - `page_size`: 每页记录数
/// - `page`: 页码
///
/// # Returns
///
/// 返回一页 [`CardTransactionDetail`]
///
/// # Errors
///
/// `token` 失效时返回 [`IPortalTokenExpired`]
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_card_transaction_records(
    token: &IPortalToken,
    account: String,
    start: NaiveDate,
    end: NaiveDate,
    page_size: u32,
    page: u32,
) -> Result<CardTransactionDetail, crate::Error<IPortalTokenExpired>> {
    let json_str =
        fetch_time!(fetch::transaction_records(token, account, start, end, page_size, page).await)?;
    let details = parse_time!(parse::card_transaction_records(&json_str))?;
    Ok(details)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    #[tokio::test]
    #[ignore]
    async fn test_get_card_info() -> crate::test::TestResult<()> {
        let token = crate::iportal::login::get_iportal_token().await?;
        let info = crate::iportal::card::get_card_info(&token).await?;
        println!("{info:#?}");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_card_transaction_records() -> crate::test::TestResult<()> {
        let token = crate::iportal::login::get_iportal_token().await?;
        let card = crate::iportal::card::get_card_info(&token).await?;
        let now = Utc::now().date_naive();
        let before_30d = now - Duration::days(30);
        let info = crate::iportal::card::get_card_transaction_records(
            &token,
            card.account,
            before_30d,
            now,
            10,
            1,
        )
        .await?;
        println!("{info:#?}");
        Ok(())
    }
}
