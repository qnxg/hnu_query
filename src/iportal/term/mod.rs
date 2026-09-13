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
#[derive(Serialize, Debug, Clone)]
pub struct TermInfo {
    /// 学期开始日期
    ///
    /// 格式为 `yyyy-MM-dd`, 例如: `2026-07-05`
    ///
    ///  **注意: 开始日期为学期第一天的 00:00:00, 不是学期第一天的 23:59:59**
    pub start_date: NaiveDateTime,
    /// 学期结束日期, **注意: 结束日期为学期最后一天的 00:00:00, 不是学期最后一天的 23:59:59**
    pub end_date: NaiveDateTime,
    /// 学期描述, 例如: `夏季`
    pub description: String,
    /// 学期编号, 秋季学期为 `1`, 寒
    pub term: TermType,
    /// 学年，例如: `2025-2026`
    pub year: String,
    /// 给定时间位于该学期的周次
    pub week: u16,
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TermType {
    /// 秋季学期
    Autumn = 1,
    /// 春季学期
    Spring = 2,
    /// 寒假
    WinterVacation = 3,
    /// 暑假
    SummerVacation = 4,
    /// 未知学期类型, 回退类型
    Other(u8),
}

/// 获取指定时间所在学期的信息
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过 [`IPortalToken::acquire_by_cas_login`] 获取
/// - `timestamp`: Unix 时间戳，单位为秒
///
/// # Returns
///
/// 返回该时间对应的 [`TermInfo`]
///
/// # Errors
///
/// 当令牌失效、网络请求失败或响应无法解析时返回错误
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_term_info(
    token: &IPortalToken,
    timestamp: i64,
) -> Result<TermInfo, crate::Error<IPortalTokenExpired>> {
    let json_str = fetch_time!(fetch::fetch_term_info(token, timestamp).await)?;
    let info = parse_time!(parse::parse_term_info(&json_str))?;
    Ok(info)
}

#[cfg(test)]
mod tests {
    use std::time::{self, UNIX_EPOCH};

    use crate::{
        iportal::{login::get_iportal_token, term::get_term_info},
        test::TestResult,
    };

    #[tokio::test]
    #[ignore]
    async fn test_fetch_term_info() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let info = get_term_info(
            &token,
            time::SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs() as i64,
        )
        .await?;
        println!("{info:#?}");
        Ok(())
    }
}
