mod fetch;
mod parse;

use crate::lab::login::LabToken;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 大物实验平台的学期信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Semester {
    /// 学年
    pub xn: u16,
    /// 学期
    pub xq: u8,
    /// 学期id
    pub id: String,
}

/// 获取大物实验平台的学期信息
///
/// # Arguments
///
/// - `lab_token`: 大物实验平台的令牌，可以通过 [LabToken::acquire_by_login] 获取
///
/// # Returns
///
/// 返回一个包含大物实验平台所有学期信息的列表
pub async fn get_semester(lab_token: &LabToken) -> Result<Vec<Semester>, crate::Error<Infallible>> {
    let json_str = fetch::semester(lab_token).await?;
    parse::semester(&json_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lab::test::get_lab_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_semester() -> TestResult<()> {
        let lab_token = get_lab_token().await?;
        let semester = get_semester(&lab_token).await?;
        println!("{:#?}", semester);
        Ok(())
    }
}
