//! 当前登录账号信息查询

use crate::iportal::error::IPortalTokenExpired;
use crate::iportal::login::IPortalToken;
use crate::utils::obs::{fetch_time, parse_time};
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
pub enum Gender {
    Male = 1,
    Female = 2,
    Other(u8),
}

mod fetch;
mod parse;

/// 获取当前登录账号的信息
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过 [`IPortalToken::acquire_by_cas_login`] 获取
///
/// # Returns
///
/// 返回包含用户、身份、联系方式及管理员状态的 [`AccountInfo`]
/// # Errors
///
/// 当令牌失效、网络请求失败或响应无法解析时返回错误
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_account_info(
    token: &IPortalToken,
) -> Result<AccountInfo, crate::Error<IPortalTokenExpired>> {
    let json_str: String = fetch_time!(fetch::fetch_info(token).await)?;
    let info = parse_time!(parse::parse_account_info(&json_str))?;
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
    async fn test_fetch_info() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let info = get_account_info(&token).await?;
        println!("{info:#?}");
        Ok(())
    }
}
