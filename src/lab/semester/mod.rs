mod raw;

use crate::{error::parse_err, lab::login::LabToken};
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
/// 返回一个包含所有大物实验平台的学期信息的列表
pub async fn get_semester(lab_token: &LabToken) -> Result<Vec<Semester>, crate::Error<Infallible>> {
    let raw_data = raw::raw_semester_data(lab_token).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let [xn_str, _, xq_str] = item
            .text
            .split(|c| ['-', '_', ' '].contains(&c))
            .collect::<Vec<&str>>()[..]
        else {
            return Err(parse_err(&item.text));
        };
        let (Ok(xn), Ok(xq)) = (xn_str.parse::<u16>(), xq_str.parse::<u8>()) else {
            return Err(parse_err(&item.text));
        };
        res.push(Semester {
            xn,
            xq,
            id: item.id,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lab::test::get_lab_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_semester() {
        let lab_token = get_lab_token().await.unwrap();
        let semester = get_semester(&lab_token).await.unwrap();
        println!("{:#?}", semester);
    }
}
