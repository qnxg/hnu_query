//! iportal 首页个人数据查询

mod fetch;
mod parse;

use crate::{
    error::MapUnexpectedErr,
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::obs::{fetch_time, parse_time},
};
use chrono::NaiveDateTime;
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

/// iportal 首页可查询的个人数据类型及其详情 ID
///
/// **注意: 该枚举的值是动态的, 可能会随时间变化而变化, 不要将其硬编码在代码中**
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PersonalData {
    /// 图书馆借阅书籍数量
    pub lib_borrow: Option<u32>,
    /// 绑定的邮箱
    pub email: Option<String>,
    /// 未读邮件数量
    pub mail_unread: Option<u32>,
    /// 校园卡余额，单位为元
    pub balance: Option<f64>,
    /// 上次登录时间
    pub last_login_time: Option<NaiveDateTime>,
    /// 校园网已用流量，单位为 Byte
    pub net_used: Option<u64>,
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
    let types_json = fetch_time!(fetch::personal_data_query_ids(token).await)?;
    let types = parse_time!(parse::personal_data_query_ids(&types_json))?;
    let mut result = PersonalData::default();
    let raw_items = fetch_time!(
        async {
            let mut tasks = JoinSet::new();
            for type_enum in types {
                let token = token.clone();
                tasks.spawn(async move {
                    let json = fetch::personal_data(&token, type_enum.clone()).await?;
                    Ok::<_, crate::Error<IPortalTokenExpired>>((type_enum, json))
                });
            }

            let mut raw_items = Vec::new();
            while let Some(task) = tasks.join_next().await {
                raw_items.push(task.unexpected_err()??);
            }
            Ok::<_, crate::Error<IPortalTokenExpired>>(raw_items)
        }
        .await
    )?;
    parse_time!({
        for (type_enum, json) in raw_items {
            let item = parse::personal_data(&json)?;
            match type_enum {
                PersonalDataTypeEnum::LibBorrow(_) => {
                    result.lib_borrow = Some(item.value.parse::<u32>().unwrap_or(0))
                }
                PersonalDataTypeEnum::MailUnread(_) => {
                    result.email = item.email.clone();
                    result.mail_unread = Some(item.value.parse::<u32>().unwrap_or(0))
                }
                PersonalDataTypeEnum::Balance(_) => {
                    result.balance = Some(item.value.parse::<f64>().unwrap_or(0.0))
                }
                PersonalDataTypeEnum::LastLoginTime(_) => {
                    if let Ok(date) =
                        NaiveDateTime::parse_from_str(&item.value, "%Y-%m-%d %H:%M:%S")
                    {
                        result.last_login_time = Some(date);
                    }
                }
                PersonalDataTypeEnum::NetUsed(_) => {
                    let value = item.value.parse::<f64>().unwrap_or(0.0);
                    let unit = item.unit.clone();
                    result.net_used = Some(parse::net_convert_to_byte(value, unit));
                }
            }
        }
    });
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{
        iportal::{
            get_personal_data_summary,
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

    #[tokio::test]
    #[ignore]
    async fn test_fetch_personal_data_summary() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let summary = get_personal_data_summary(&token).await?;
        println!("{summary:#?}");
        Ok(())
    }
}
