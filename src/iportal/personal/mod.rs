//! iportal 首页个人数据查询

mod fetch;
mod parse;

use crate::{
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::obs::{fetch_time, parse_time},
};
use hnu_query_macros::traced;

/// iportal 首页可查询的个人数据类型及其详情 ID
///
/// **注意: 该枚举的值是动态的, 可能会随时间变化而变化, 不要将其硬编码在代码中**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalDataTypeEnum {
    /// 图书馆借阅数量
    LibBorrow(String),
    /// 未读邮件数量
    MailUnread(String),
    /// 校园卡余额
    Balance(String),
    /// 上次登录时间
    LastLoginTime(String),
    /// 校园网已用流量
    NetUsed(String),
}

impl PersonalDataTypeEnum {
    /// 获取个人数据类型的查询 ID
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(self), fields(subsystem = "iportal"))
    )]
    pub fn get_value(self) -> String {
        match self {
            Self::LibBorrow(value)
            | Self::MailUnread(value)
            | Self::Balance(value)
            | Self::LastLoginTime(value)
            | Self::NetUsed(value) => value,
        }
    }
}

/// 一项个人数据的详情
#[derive(Debug, Clone)]
pub struct PersonalDataItem {
    /// 数据值
    pub value: String,
    /// 数据单位；服务端未提供单位时为 `None`, 提供时可能为 `"元"`、`"GB"` 等
    pub unit: Option<String>,
    /// 数据项名称
    pub name: String,
    /// 与该数据项相关的邮箱；不适用或服务端未提供时为 `None`
    pub email: Option<String>,
}

/// 当前账号首页个人数据的聚合结果。
///
/// 服务端未返回的项目为 `None`。
#[derive(Debug, Clone, Default)]
pub struct PersonalData {
    pub lib_borrow: Option<PersonalDataItem>,
    pub mail_unread: Option<PersonalDataItem>,
    pub balance: Option<PersonalDataItem>,
    pub last_login_time: Option<PersonalDataItem>,
    pub net_used: Option<PersonalDataItem>,
}

/// 获取当前账号可查询的个人数据类型及其详情 ID
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过 [`IPortalToken::acquire_by_cas_login`] 获取
///
/// # Returns
///
/// 返回的个人数据类型中的值可以传给 [`get_personal_data`] 获取详情
///
/// # Errors
///
/// `token` 失效时返回 [`IPortalTokenExpired`]
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_personal_data_lists(
    token: &IPortalToken,
) -> Result<Vec<PersonalDataTypeEnum>, crate::Error<IPortalTokenExpired>> {
    let json_str = fetch_time!(fetch::personal_data_query_ids(token).await)?;
    let items = parse_time!(parse::personal_data_query_ids(&json_str))?;
    Ok(items)
}

/// 获取一项个人数据的详情
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过 [`IPortalToken::acquire_by_cas_login`] 获取
/// - `type_enum`: 从 [`get_personal_data_lists`] 获得的数据类型及详情 ID
///
/// # Returns
///
/// 返回对应的 [`PersonalDataItem`]
///
/// # Errors
///
/// `token` 失效时返回 [`IPortalTokenExpired`]
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_personal_data(
    token: &IPortalToken,
    type_enum: PersonalDataTypeEnum,
) -> Result<PersonalDataItem, crate::Error<IPortalTokenExpired>> {
    let json_str = fetch_time!(fetch::personal_data(token, type_enum.clone()).await)?;
    let item = parse_time!(parse::personal_data(&json_str))?;
    Ok(item)
}

/// 获取当前账号全部可用的个人数据。
///
/// 该函数会先获取数据项列表，再逐项获取详情；服务端未提供的项目保留为 `None`。
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_personal_data_summary(
    token: &IPortalToken,
) -> Result<PersonalData, crate::Error<IPortalTokenExpired>> {
    let types = get_personal_data_lists(token).await?;
    let mut result = PersonalData::default();
    for type_enum in types {
        let item = get_personal_data(token, type_enum.clone()).await?;
        match type_enum {
            PersonalDataTypeEnum::LibBorrow(_) => result.lib_borrow = Some(item),
            PersonalDataTypeEnum::MailUnread(_) => result.mail_unread = Some(item),
            PersonalDataTypeEnum::Balance(_) => result.balance = Some(item),
            PersonalDataTypeEnum::LastLoginTime(_) => result.last_login_time = Some(item),
            PersonalDataTypeEnum::NetUsed(_) => result.net_used = Some(item),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{
        iportal::{
            login::get_iportal_token,
            personal::{get_personal_data, get_personal_data_lists},
        },
        test::TestResult,
    };

    #[tokio::test]
    #[ignore]
    async fn test_fetch_personal_data_query_ids() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let ids = get_personal_data_lists(&token).await?;
        println!("{ids:#?}");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_personal_data() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let ids = get_personal_data_lists(&token).await?;
        for id in ids {
            let item = get_personal_data(&token, id.clone()).await?;
            println!("{id:?}: {item:#?}");
        }
        Ok(())
    }
}
