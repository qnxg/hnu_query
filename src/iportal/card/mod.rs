//! 校园卡账户与交易明细查询。

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};

use crate::utils::obs::{fetch_time, parse_time};

mod fetch;
mod parse;

/// 校园卡账户信息。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardInfo {
    /// 校园卡账号。
    pub account: String,
    /// 校园卡余额。
    pub balance: f64,
}

/// 一条校园卡交易记录。
///
/// 交易金额由 iPortal 以人民币分为单位的字符串返回。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardFlowInfoItem {
    /// 商户名称。
    #[serde(rename = "mercname")]
    pub merchant_name: String,
    /// 商户账号。
    #[serde(rename = "mercacc")]
    pub merchant_account: String,
    /// 交易时间。
    ///
    /// iPortal 返回的北京时间会转换为 UTC。
    #[serde(rename = "occtime", deserialize_with = "parse_time_fn")]
    pub pay_time: DateTime<Utc>,
    /// 带收支方向的交易金额，单位为分。
    #[serde(rename = "sign_tranamt")]
    pub sign_trans_amount: String,
    /// 交易金额，单位为分。
    #[serde(rename = "tranamt")]
    pub trans_amount: String,
    /// 交易类型代码。
    #[serde(rename = "trancode")]
    pub trans_code: String,
    /// 交易类型名称。
    #[serde(rename = "tranname")]
    pub trans_name: String,
}

/// 分页的校园卡交易明细。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardDetailsItem {
    /// 下一页页码；`"0"` 表示没有下一页。
    #[serde(rename = "nextpage")]
    pub next_page: String,
    /// 每页记录数。
    #[serde(rename = "pagesize")]
    pub page_size: String,
    /// 符合查询条件的记录总数。
    #[serde(rename = "rowcount")]
    pub row_count: String,
    /// 当前页的交易记录。
    #[serde(rename = "total")]
    pub data: Vec<CardFlowInfoItem>,
}

fn parse_time_fn<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt = NaiveDateTime::parse_from_str(&s, "%Y%m%d%H%M%S").map_err(serde::de::Error::custom)?;
    let timezone = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| serde::de::Error::custom("invalid timezone"))?;
    let dt = timezone
        .from_local_datetime(&dt)
        .single()
        .ok_or_else(|| serde::de::Error::custom("invalid datetime"))?;
    Ok(dt.to_utc())
}

/// 获取校园卡账号及余额。
///
/// # Arguments
///
/// - `token`: 个人门户令牌，可以通过
///   [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login)
///   获取
///
/// # Returns
///
/// 返回 [`CardInfo`]。
///
/// # Errors
///
/// 当令牌失效、网络请求失败或响应无法解析时返回错误。
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_card_info(
    token: &crate::iportal::login::IPortalToken,
) -> Result<crate::iportal::card::CardInfo, crate::Error<crate::cas::error::TokenExpired>> {
    let json_str = fetch_time!(fetch::fetch_balance(token).await)?;
    let balance = parse_time!(parse::parse_card_balance(&json_str))?;
    Ok(balance)
}

/// 分页获取指定日期范围内的校园卡交易明细。
///
/// # Arguments
///
/// - `token`: 个人门户令牌，可以通过
///   [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login)
///   获取
/// - `start`: 查询开始时间；请求时只使用 UTC 日期部分
/// - `end`: 查询结束时间；请求时只使用 UTC 日期部分
/// - `pagesize`: 每页记录数，`None` 时为 10
/// - `page`: 页码，`None` 时为 1
/// - `account`: 校园卡账号，可以通过 [`get_card_info`] 获取
///
/// # Returns
///
/// 返回一页 [`CardDetailsItem`]；其中交易时间为 UTC，`next_page` 为 `"0"`
/// 表示没有下一页。
///
/// # Errors
///
/// 当令牌失效、网络请求失败或响应无法解析时返回错误。
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_card_details(
    token: &crate::iportal::login::IPortalToken,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    pagesize: Option<u32>,
    page: Option<u32>,
    account: &str,
) -> Result<crate::iportal::card::CardDetailsItem, crate::Error<crate::cas::error::TokenExpired>> {
    let json_str =
        fetch_time!(fetch::fetch_card_details(token, start, end, pagesize, page, account).await)?;
    let details = parse_time!(parse::parse_card_details(&json_str))?;
    Ok(details)
}

#[cfg(test)]
mod tests {
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
    async fn test_get_card_details() -> crate::test::TestResult<()> {
        let token = crate::iportal::test::get_iportal_token().await?;
        let card = crate::iportal::card::get_card_info(&token).await?;
        let info = crate::iportal::card::get_card_details(
            &token,
            chrono::Utc::now() - chrono::Duration::days(30),
            chrono::Utc::now(),
            Some(10),
            Some(1),
            card.account.as_str(),
        )
        .await?;
        println!("{info:#?}");
        Ok(())
    }
}
