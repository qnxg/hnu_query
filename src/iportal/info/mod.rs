//! 当前登录账号信息查询

use crate::iportal::error::IPortalExpired;
use crate::iportal::login::IPortalToken;
use crate::utils::obs::{fetch_time, parse_time};
use chrono::NaiveDateTime;
use hnu_query_macros::traced;
use serde::{Deserialize, Deserializer, Serialize};

/// 当前登录账号的信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountInfo {
    /// 用户 ID
    pub uid: String,
    /// 姓名
    pub name: String,
    /// 学工号
    pub xgh: String,
    /// 当前身份名称
    pub identity: String,
    /// 当前身份 ID
    pub identity_id: String,
    /// 性别代码
    pub sex: u8,
    /// 所属部门
    pub depart: String,
    /// 手机号码
    pub mobile: String,
    /// 电子邮箱
    pub email: String,
    // organ: HashMap<String, u8>,
    /// 头像地址
    pub avatar: String,
    /// 登录时间
    #[serde(deserialize_with = "deserialize_naive_datetime")]
    pub time: NaiveDateTime,
    /// 是否为系统管理员
    pub is_manager: bool,
    /// 是否为应用管理员
    pub is_app_manager: bool,
    /// 是否为流程管理员
    pub is_process_manager: bool,
    //user_config: Option<serde_json::Value>,
}

fn deserialize_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S")
        .map_err(serde::de::Error::custom)
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
) -> Result<AccountInfo, crate::Error<IPortalExpired>> {
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
