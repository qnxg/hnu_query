//! 当前登录账号信息查询

mod fetch;
mod parse;

use crate::{
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::obs::{fetch_time, parse_time},
};
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};

/// 当前登录账号的信息
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccountInfo {
    /// iportal 用户唯一标识，例如：`"114514"`
    pub uid: String,
    /// 姓名
    pub name: String,
    /// 学工号；学生通常为学号，教职工通常为工号，例如：`"191908100721"`
    pub id: String,
    /// 当前身份名称，例如: `本科生`
    pub identity: String,
    /// 当前身份在 iportal 中的内部 ID，例如：身份名称为 `本科生` 时可能为 `"2002"`
    pub identity_id: u64,
    /// 性别代码。具体代码含义由 iportal 服务端定义
    pub gender: Gender,
    /// 所属学院、部门或其他组织名称，例如：`"计算机学院"`
    pub depart: String,
    /// 头像 URL；服务端未提供时可能为空字符串
    pub avatar: String,
}

#[repr(u8)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// 性别
pub enum Gender {
    /// 男性
    Male = 1,
    /// 女性
    Female = 2,
    /// 服务端返回的其他性别代码
    Other(u8),
}

/// 获取当前登录账号的信息
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过 [`IPortalToken::acquire_by_cas_login`] 获取
///
/// # Returns
///
/// 返回当前登录账号的 [`AccountInfo`]
///
/// # Errors
///
/// `token` 失效时返回 [`IPortalTokenExpired`]
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_account_info(
    token: &IPortalToken,
) -> Result<AccountInfo, crate::Error<IPortalTokenExpired>> {
    let json_str: String = fetch_time!(fetch::info(token).await)?;
    let info = parse_time!(parse::account_info(&json_str))?;
    Ok(info)
}

#[cfg(test)]
mod tests {
    use crate::{
        iportal::{info::get_account_info, login::get_iportal_token},
        test::TestResult,
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_account_info() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let info = get_account_info(&token).await?;
        println!("{info:#?}");
        Ok(())
    }
}
