//! 学年、学期与周次查询。

mod fetch;
mod parse;

use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};

use crate::{
    iportal::login::IPortalToken,
    utils::obs::{fetch_time, parse_time},
};

/// 给定时间所在学期的信息。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TermInfo {
    /// 学期开始日期。
    pub start_date: String,
    /// 学期结束日期。
    pub end_date: String,
    /// 学期描述。
    #[serde(rename = "dsc")]
    pub description: String,
    /// 学期。
    pub term: String,
    /// 学年。
    pub year: String,
    /// 给定时间位于该学期的周次。
    pub week: u16,
}

/// 获取指定时间所在学期的信息。
///
/// # Arguments
///
/// - `token`: 个人门户令牌，可以通过 [`IPortalToken::acquire_by_cas_login`] 获取
/// - `timestamp`: Unix 时间戳，单位为秒
///
/// # Returns
///
/// 返回该时间对应的 [`TermInfo`]。
///
/// # Errors
///
/// 当令牌失效、网络请求失败或响应无法解析时返回错误。
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_term_info(
    token: &IPortalToken,
    timestamp: i64,
) -> Result<TermInfo, crate::Error<crate::cas::error::TokenExpired>> {
    let json_str = fetch_time!(fetch::fetch_term_info(token, timestamp).await)?;
    let info = parse_time!(parse::parse_term_info(&json_str))?;
    Ok(info)
}

#[cfg(test)]
mod tests {
    use std::time::{self, UNIX_EPOCH};

    use crate::{
        iportal::{term::get_term_info, test::get_iportal_token},
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
