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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// 交易类型
pub enum TransactionType {
    /// 拍卡消费
    Consumption,
    /// 扫码支付
    QRCodeConsumption,
    /// 充值
    Recharge,
    /// 领取补助
    ///
    /// 部分交易不会直接写卡, 需要在校内相应终端上刷卡才会实际同步
    ///
    /// 此交易类型仅为同步记录, 不代表余额增减, 因此从记录推算消费流水时应将此项排除
    ///
    /// 例如：先充值 100 元, 随后使用洗衣机消费 3 元
    ///
    /// 这两次操作会分别产生一条 100 元的 `Recharge` 记录和三条 -1 元的 `Withholding` 记录
    ///
    /// 在终端刷卡同步余额后，还会产生一条 97 元的 `MoneyTransfer` 记录
    ///
    /// 该记录只是将尚未写卡的 97 元同步到校园卡，并不表示又产生了 97 元收入
    MoneyTransfer,
    /// 补助
    Benefit,
    /// 电子账户开户, 出现于新生开卡
    EAccountOpening,
    /// 持卡人开户, 出现于新生开卡
    HolderAccountOpening,
    /// 代扣代缴, 出现于洗衣机/烘干机收费
    Withholding,
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
/// - `token`: iportal 令牌, 可以通过 [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login) 获取
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
/// - `account`: 校园卡账号，可以通过 [`get_card_info`] 获取
/// - `start`: 查询开始日期
/// - `end`: 查询结束日期
/// - `page_size`: 每页记录数
/// - `page`: 页码
///
/// # Preconditions
///
/// 需要确保传入的 `account` 是使用同一个 `token` 调用 [`get_card_info`] 得到的，否则会出现未定义行为。
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
        let token = crate::iportal::test::get_iportal_token().await?;
        let info = crate::iportal::card::get_card_info(&token).await?;
        println!("{info:#?}");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_card_transaction_records() -> crate::test::TestResult<()> {
        let token = crate::iportal::test::get_iportal_token().await?;
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
