use crate::{
    error::{MapParseErr, parse_err},
    iportal::{
        error::IPortalTokenExpired,
        personal::{PersonalDataItem, PersonalDataTypeEnum},
        util::iportal_jsondata_precheck,
    },
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "key", content = "id")]
enum RawPersonalDataType {
    #[serde(rename = "book.bookNum")]
    LibBorrow(String),
    #[serde(rename = "mail.unread")]
    MailUnread(String),
    #[serde(rename = "card.balance")]
    Balance(String),
    #[serde(rename = "statistic.lastLoginTime")]
    LastLoginTime(String),
    #[serde(rename = "net.used")]
    NetUsed(String),
}

#[derive(Debug, Deserialize)]
struct RawPersonalDataItem {
    value: serde_json::Value,
    unit: Option<String>,
    #[serde(alias = "title")]
    name: String,
    email: Option<String>,
}

/// 解析当前账号可查询的个人数据类型响应
///
/// # Arguments
///
/// - `json_str`: [`super::fetch::personal_data_query_ids`] 返回的数据
pub fn personal_data_query_ids(
    json_str: &str,
) -> Result<Vec<PersonalDataTypeEnum>, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str)?;
    json_value
        .get("data")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| parse_err("无法解析 data 字段为数组", json_str))?
        .iter()
        .cloned()
        .map(|item| {
            let raw: RawPersonalDataType = serde_json::from_value(item).parse_err(json_str)?;
            Ok(match raw {
                RawPersonalDataType::LibBorrow(v) => PersonalDataTypeEnum::LibBorrow(v),
                RawPersonalDataType::MailUnread(v) => PersonalDataTypeEnum::MailUnread(v),
                RawPersonalDataType::Balance(v) => PersonalDataTypeEnum::Balance(v),
                RawPersonalDataType::LastLoginTime(v) => PersonalDataTypeEnum::LastLoginTime(v),
                RawPersonalDataType::NetUsed(v) => PersonalDataTypeEnum::NetUsed(v),
            })
        })
        .collect()
}

/// 解析一项个人数据的详情响应
///
/// # Arguments
///
/// - `json_str`: [`super::fetch::personal_data`] 返回的数据
pub fn personal_data(
    json_str: &str,
) -> Result<PersonalDataItem, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str)?;
    let data_item = json_value
        .get("data")
        .ok_or_else(|| parse_err("无法解析 data 字段", json_str))?
        .clone();
    let raw: RawPersonalDataItem = serde_json::from_value(data_item).parse_err(json_str)?;
    let value = match raw.value {
        serde_json::Value::String(v) => v,
        serde_json::Value::Number(v) => v.to_string(),
        _ => return Err(parse_err("无法解析 value 字段", json_str)),
    };
    Ok(PersonalDataItem {
        value,
        unit: raw.unit,
        name: raw.name,
        email: raw.email,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_personal_data_query_ids() -> TestResult<()> {
        let items = personal_data_query_ids(include_str!("test_data/ids.json"))?;

        assert_eq!(items.len(), 5);
        assert!(matches!(
            &items[0],
            PersonalDataTypeEnum::LibBorrow(id)
                if id == "00c7f05e1561d58c0948743b459a248e582a99ff8e07f782e53b1c9feda8208388bc1c82afbabcb8ced7f359778f47a8da48d793effc8f440e9f0bf56f164fa318befcf401062f25a64a11e0b56eaaadf21b57ddc1ea476b15daa3cc4a66cc6117f9d375720c3b32bca811393e52fe32add9dfc61e243a3d6d90fa02da8d6eb1"
        ));
        assert!(matches!(&items[1], PersonalDataTypeEnum::MailUnread(id) if id == "1919810"));
        assert!(matches!(&items[2], PersonalDataTypeEnum::Balance(id) if id == "114514"));
        assert!(matches!(&items[3], PersonalDataTypeEnum::LastLoginTime(id) if id == "168"));
        assert!(matches!(&items[4], PersonalDataTypeEnum::NetUsed(id) if id == "0d000721"));

        Ok(())
    }

    #[test]
    fn test_parse_personal_data_card() -> TestResult<()> {
        let card = personal_data(include_str!("test_data/id_data/card.json"))?;
        assert_eq!(card.name, "一卡通余额");
        assert_eq!(card.value, "81.81");
        assert_eq!(card.unit, Some("元".to_string()));
        assert_eq!(card.email, None);
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_email() -> TestResult<()> {
        let email = personal_data(include_str!("test_data/id_data/email.json"))?;
        assert_eq!(email.name, "未读邮件");
        assert_eq!(email.value, "1");
        assert_eq!(email.unit, None);
        assert_eq!(email.email, Some("admin@hnu.edu.cn".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_last_login() -> TestResult<()> {
        let last_login = personal_data(include_str!("test_data/id_data/lastlogin.json"))?;
        assert_eq!(last_login.name, "最近一次登录时间");
        assert_eq!(last_login.value, "2077-06-15 16:04:00");
        assert_eq!(last_login.unit, None);
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_lib() -> TestResult<()> {
        let lib = personal_data(include_str!("test_data/id_data/lib.json"))?;
        assert_eq!(lib.name, "待还图书");
        assert_eq!(lib.value, "0");
        assert_eq!(lib.unit, Some("".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_net() -> TestResult<()> {
        let net = personal_data(include_str!("test_data/id_data/net.json"))?;
        assert_eq!(net.name, "流量查询");
        assert_eq!(net.value, "114514");
        assert_eq!(net.unit, Some("G".to_string()));

        Ok(())
    }
}
