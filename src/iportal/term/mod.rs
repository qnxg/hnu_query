//! 学年、学期与周次查询

mod fetch;
mod parse;

use crate::{
    iportal::{error::IPortalExpired, login::IPortalToken},
    utils::obs::{fetch_time, parse_time},
};
use chrono::NaiveDateTime;
use hnu_query_macros::traced;
use serde::{Deserialize, Deserializer, Serialize};

/// 学期信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TermInfo {
    /// 学期开始日期
    #[serde(deserialize_with = "deserialize_date_as_naive_datetime")]
    pub start_date: NaiveDateTime,
    /// 学期结束日期
    #[serde(deserialize_with = "deserialize_date_as_naive_datetime")]
    pub end_date: NaiveDateTime,
    /// 学期描述
    #[serde(rename = "dsc")]
    pub description: String,
    /// 学期
    pub term: String,
    /// 学年
    pub year: String,
    /// 给定时间位于该学期的周次
    pub week: u16,
}

fn deserialize_date_as_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&format!("{value} 00:00:00"), "%Y-%m-%d %H:%M:%S")
        .map_err(serde::de::Error::custom)
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
) -> Result<TermInfo, crate::Error<IPortalExpired>> {
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
