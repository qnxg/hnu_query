//! iPortal 首页个人数据查询

use hnu_query_macros::traced;
use serde::{Deserialize, Deserializer};

use crate::{
    iportal::{error::IPortalExpired, login::IPortalToken},
    utils::obs::{fetch_time, parse_time},
};

mod fetch;
mod parse;

/// iPortal 首页可查询的个人数据类型及其详情 ID
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "key", content = "id")]
pub enum PersonalDataTypeEnum {
    /// 图书馆借阅数量
    #[serde(rename = "book.bookNum")]
    LibBorrow(String),
    /// 未读邮件数量
    #[serde(rename = "mail.unread")]
    MailUnread(String),
    /// 校园卡余额
    #[serde(rename = "card.balance")]
    Balance(String),
    /// 上次登录时间
    #[serde(rename = "statistic.lastLoginTime")]
    LastLoginTime(String),
    /// 校园网已用流量
    #[serde(rename = "net.used")]
    NetUsed(String),
}

impl PersonalDataTypeEnum {
    pub fn into_value(self) -> String {
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
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PersonalDataItem {
    /// 数据值
    #[serde(deserialize_with = "deserialize_string_or_float")]
    pub value: String,
    /// 数据单位；服务端未提供单位时为 `None`
    pub unit: Option<String>,
    /// 数据项名称
    #[serde(alias = "title")]
    pub name: String,
    /// 与该数据项相关的邮箱；不适用或服务端未提供时为 `None`
    pub email: Option<String>,
}

fn deserialize_string_or_float<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrFloat {
        String(String),
        Float(f64),
    }

    match StringOrFloat::deserialize(deserializer)? {
        StringOrFloat::String(v) => Ok(v),
        StringOrFloat::Float(v) => Ok(v.to_string()),
    }
}

/// 获取当前账号可查询的个人数据类型及其详情 ID
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过 [`IPortalToken::acquire_by_cas_login`] 获取
///
/// # Returns
///
/// 返回个人数据类型列表列表中的值可以传给 [`get_personal_data`] 获取详情
///
/// # Errors
///
/// 当令牌失效、网络请求失败或响应无法解析时返回错误
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_personal_data_lists(
    token: &IPortalToken,
) -> Result<Vec<PersonalDataTypeEnum>, crate::Error<IPortalExpired>> {
    let json_str = fetch_time!(fetch::fetch_personal_data_query_ids(token).await)?;
    let items = parse_time!(parse::parse_personal_data_query_ids(&json_str))?;
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
/// 当令牌失效、网络请求失败或响应无法解析时返回错误
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_personal_data(
    token: &IPortalToken,
    type_enum: PersonalDataTypeEnum,
) -> Result<PersonalDataItem, crate::Error<IPortalExpired>> {
    let json_str = fetch_time!(fetch::fetch_personal_data(token, type_enum.clone()).await)?;
    let item = parse_time!(parse::parse_personal_data(&json_str))?;
    Ok(item)
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
