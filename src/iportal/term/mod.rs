//! 学年、学期与周次查询

mod fetch;
mod parse;

use crate::{
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::obs::{fetch_time, parse_time},
};
use chrono::NaiveDateTime;
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};

/// 学期信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TermInfo {
    /// 学期开始日期
    pub start_date: NaiveDateTime,
    /// 学期结束日期
    pub end_date: NaiveDateTime,
    /// 学期编号
    pub term: TermType,
    /// 学年，例如: `2025-2026`
    pub year: String,
    /// 给定时间位于该学期的周次
    pub week: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// 学期类型
pub enum TermType {
    /// 秋季学期
    Autumn = 1,
    /// 春季学期
    Spring = 3,
    /// 寒假
    WinterVacation = 2,
    /// 暑假
    SummerVacation = 4,
}

/// 获取指定时间所在学期的信息
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过 [`IPortalToken::acquire_by_cas_login`] 获取
/// - `timestamp`: 指定的查询对应时间
///
/// # Returns
///
/// 返回该时间对应的学期信息
///
/// # Errors
///
/// `token` 失效时返回 [`IPortalTokenExpired`]
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_term_info(
    token: &IPortalToken,
    timestamp: NaiveDateTime,
) -> Result<TermInfo, crate::Error<IPortalTokenExpired>> {
    let timestamp = timestamp.and_utc().timestamp();
    let json_str = fetch_time!(fetch::term_info(token, timestamp).await)?;
    let info = parse_time!(parse::term_info(&json_str))?;
    Ok(info)
}

#[cfg(test)]
mod tests {
    use crate::{
        iportal::{login::get_iportal_token, term::get_term_info},
        test::TestResult,
    };

    #[tokio::test]
    #[ignore]
    async fn test_fetch_term_info() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let info = get_term_info(&token, chrono::Utc::now().naive_utc()).await?;
        println!("{info:#?}");
        Ok(())
    }
}
