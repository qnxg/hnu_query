mod fetch;
mod parse;

use crate::{
    utils::obs::{fetch_time, parse_time, traced},
    yjsxt::{error::TokenExpired, login::YjsxtToken},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Semester {
    /// 学年
    pub xn: u16,
    /// 学期
    pub xq: u8,
    /// 学期id
    pub id: String,
}

/// 获取研究生系统的学期信息
///
/// # Arguments
///
/// - `yjsxt_token`: 研究生系统的令牌，可以通过 [YjsxtToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 返回一个包含研究生系统所有学期信息的列表
#[traced(subsystem = "yjsxt", skip(yjsxt_token))]
pub async fn get_semester(
    yjsxt_token: &YjsxtToken,
) -> Result<Vec<Semester>, crate::Error<TokenExpired>> {
    let json_str = fetch_time!(fetch::semester(yjsxt_token).await)?;
    parse_time!(parse::semester(&json_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::TestResult, yjsxt::test::get_yjsxt_token};

    #[tokio::test]
    #[ignore]
    async fn test_get_semester() -> TestResult<()> {
        let yjsxt_token = get_yjsxt_token().await?;
        let semesters = get_semester(&yjsxt_token).await?;
        println!("{:?}", semesters);
        Ok(())
    }
}
