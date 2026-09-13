//! 校园卡账户与交易明细查询

use chrono::NaiveDateTime;
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};

use crate::iportal::error::IPortalTokenExpired;
use crate::utils::obs::{fetch_time, parse_time};

mod fetch;
mod parse;

/// 校园卡账户信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardInfo {
    /// 校园卡账号
    pub account: String,
    /// 校园卡余额
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
    /// 交易时间, 格式为 `yyyy-MM-dd HH:mm:ss`
    pub pay_time: NaiveDateTime,
    /// 交易金额, 正值为收入，负值为支出，单位为分
    pub transcation_amount: i64,
    /// 交易类型
    pub transaction_type: TransactionType,
}

#[repr(u32)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TransactionType {
    /// 消费
    Consumption = 15,
    /// 充值
    Recharge = 16,
    /// 领取补助 (电子账户->校园卡)
    TransferMoneyToCard = 22,
    /// 补助
    Benefit = 17,
    /// 电子账户开户
    EAccountOpening = 6,
    /// 持卡人开户
    HolderAccountOpening = 1,
    /// 其他
    Other(u32),
}

/// 分页的校园卡交易明细
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardTransactionDetail {
    /// 下一页页码; `0` 表示没有下一页
    pub next_page: u32,
    /// 每页记录数
    pub page_size: u32,
    /// 符合查询条件的记录总数
    pub row_count: u32,
    /// 当前页的交易记录
    pub records: Vec<TransactionRecord>,
}

/// 获取校园卡账号及余额
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过
///   [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login)
///   获取
///
/// # Returns
///
/// 返回 [`CardInfo`]
///
/// # Errors
///
/// 当令牌失效、网络请求失败或响应无法解析时返回错误
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_card_balance_info(
    token: &crate::iportal::login::IPortalToken,
) -> Result<crate::iportal::card::CardInfo, crate::Error<IPortalTokenExpired>> {
    let json_str = fetch_time!(fetch::fetch_balance(token).await)?;
    let balance = parse_time!(parse::parse_card_balance(&json_str))?;
    Ok(balance)
}

/// 分页获取指定日期范围内的校园卡交易明细
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过
///   [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login)
///   获取
/// - `start`: 查询开始时间, 格式为 `yyyy-MM-dd`
/// - `end`: 查询结束时间, 格式为 `yyyy-MM-dd`
/// - `pagesize`: 每页记录数，`None` 时为 10
/// - `page`: 页码，`None` 时为 1
/// - `account`: 校园卡账号，可以通过 [`get_card_balance_info`] 获取
///
/// # Returns
///
/// 返回一页 [`CardTransactionDetail`]；其中交易时间为 UTC，`next_page` 为 `"0"`
/// 表示没有下一页
///
/// # Errors
///
/// 当令牌失效、网络请求失败或响应无法解析时返回错误
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_card_transaction_records(
    token: &crate::iportal::login::IPortalToken,
    start: NaiveDateTime,
    end: NaiveDateTime,
    pagesize: Option<u32>,
    page: Option<u32>,
    account: &str,
) -> Result<crate::iportal::card::CardTransactionDetail, crate::Error<IPortalTokenExpired>> {
    let json_str = fetch_time!(
        fetch::fetch_card_transaction_records(token, start, end, pagesize, page, account).await
    )?;
    let details = parse_time!(parse::parse_card_transaction_records(&json_str))?;
    Ok(details)
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::FixedOffset;

    #[tokio::test]
    #[ignore]
    async fn test_get_card_balance_info() -> crate::test::TestResult<()> {
        let token = crate::iportal::login::get_iportal_token().await?;
        let info = crate::iportal::card::get_card_balance_info(&token).await?;
        println!("{info:#?}");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_card_transaction_records() -> crate::test::TestResult<()> {
        let token = crate::iportal::login::get_iportal_token().await?;
        let card = crate::iportal::card::get_card_balance_info(&token).await?;
        let info = crate::iportal::card::get_card_transaction_records(
            &token,
            chrono::Utc::now()
                .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())
                .add(-chrono::Duration::days(30))
                .naive_local(),
            chrono::Utc::now()
                .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())
                .naive_local(),
            Some(10),
            Some(1),
            card.account.as_str(),
        )
        .await?;
        println!("{info:#?}");
        Ok(())
    }
}
