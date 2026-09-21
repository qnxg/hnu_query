use crate::{
    error::{MapParseErr, parse_err},
    iportal::{
        error::IPortalTokenExpired, personal::PersonalData, util::iportal_jsondata_precheck,
    },
};
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawPersonalDataType {
    key: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct RawPersonalDataItem {
    #[serde(deserialize_with = "deserialize_value")]
    value: String,
    unit: Option<String>,
    #[serde(rename = "name", alias = "title")]
    _name: String,
    email: Option<String>,
}

fn deserialize_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        _ => Err(serde::de::Error::custom("value 必须是字符串或数字")),
    }
}

/// 解析当前账号可查询的个人数据类型响应
///
/// # Arguments
///
/// - `json_str`: [`super::fetch::personal_data_query_ids`] 返回的数据
pub fn personal_data_query_ids(
    json_str: &str,
) -> Result<Vec<(String, String)>, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str)?;
    json_value
        .get("data")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| parse_err("无法解析 data 字段为数组", json_str))?
        .iter()
        .cloned()
        .map(|item| {
            let raw: RawPersonalDataType = serde_json::from_value(item).parse_err(json_str)?;
            if !is_supported_key(&raw.key) {
                return Err(parse_err(
                    &format!("未知个人数据类型: {}", raw.key),
                    json_str,
                ));
            }
            Ok((raw.key, raw.id))
        })
        .collect()
}

/// 解析一项个人数据的详情响应
///
/// # Arguments
///
/// - `json_str`: [`super::fetch::personal_data_single`] 返回的数据
fn personal_data_single(
    json_str: &str,
) -> Result<RawPersonalDataItem, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str)?;
    let data_item = json_value
        .get("data")
        .ok_or_else(|| parse_err("无法解析 data 字段", json_str))?
        .clone();
    serde_json::from_value(data_item).parse_err(json_str)
}

/// 解析并聚合所有个人数据详情响应。
///
/// `items` 中的 key 和 id 来自 [`personal_data_query_ids`]，详情 JSON 来自
/// [`super::fetch::personal_data_single`]。
pub fn personal_data(
    items: impl IntoIterator<Item = (String, String)>,
) -> Result<PersonalData, crate::Error<IPortalTokenExpired>> {
    let mut result = PersonalData::default();
    for (key, json_str) in items {
        let item = personal_data_single(&json_str)?;
        let value = item.value;
        match key.as_str() {
            "book.bookNum" => result.lib_borrow = Some(value.parse::<u32>().parse_err(&json_str)?),
            "mail.unread" => {
                result.email = item.email;
                result.mail_unread = Some(value.parse::<u32>().parse_err(&json_str)?);
            }
            "card.balance" => result.balance = Some(value.parse::<f64>().parse_err(&json_str)?),
            "statistic.lastLoginTime" => {
                if let Ok(date) = NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S") {
                    result.last_login_time = Some(date);
                }
            }
            "net.used" => {
                let value = value.parse::<f64>().unwrap_or(0.0);
                result.net_used = Some(net_convert_to_byte(value, item.unit));
            }
            _ => return Err(parse_err(&format!("未知个人数据类型: {key}"), &json_str)),
        }
    }
    Ok(result)
}

fn is_supported_key(key: &str) -> bool {
    matches!(
        key,
        "book.bookNum" | "mail.unread" | "card.balance" | "statistic.lastLoginTime" | "net.used"
    )
}

fn net_convert_to_byte(value: f64, unit: Option<String>) -> u64 {
    match unit.map(|s| s.to_uppercase().trim().to_string()).as_deref() {
        Some("GB") | Some("G") => (value * 1024.0 * 1024.0 * 1024.0) as u64,
        Some("MB") | Some("M") => (value * 1024.0 * 1024.0) as u64,
        Some("KB") | Some("K") => (value * 1024.0) as u64,
        _ => value as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_personal_data_query_ids() -> TestResult<()> {
        let items = personal_data_query_ids(include_str!("test_data/ids.json"))?;

        assert_eq!(items.len(), 5);
        assert_eq!(
            items[0],
            (
                "book.bookNum".to_string(),
                "00c7f05e1561d58c0948743b459a248e582a99ff8e07f782e53b1c9feda8208388bc1c82afbabcb8ced7f359778f47a8da48d793effc8f440e9f0bf56f164fa318befcf401062f25a64a11e0b56eaaadf21b57ddc1ea476b15daa3cc4a66cc6117f9d375720c3b32bca811393e52fe32add9dfc61e243a3d6d90fa02da8d6eb1".to_string(),
            )
        );
        assert_eq!(items[1], ("mail.unread".to_string(), "1919810".to_string()));
        assert_eq!(items[2], ("card.balance".to_string(), "114514".to_string()));
        assert_eq!(
            items[3],
            ("statistic.lastLoginTime".to_string(), "168".to_string())
        );
        assert_eq!(items[4], ("net.used".to_string(), "0d000721".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_personal_data_card() -> TestResult<()> {
        let card = personal_data_single(include_str!("test_data/id_data/card.json"))?;
        assert_eq!(card._name, "一卡通余额");
        assert_eq!(card.value, "81.81");
        assert_eq!(card.unit, Some("元".to_string()));
        assert_eq!(card.email, None);
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_email() -> TestResult<()> {
        let email = personal_data_single(include_str!("test_data/id_data/email.json"))?;
        assert_eq!(email._name, "未读邮件");
        assert_eq!(email.value, "1");
        assert_eq!(email.unit, None);
        assert_eq!(email.email, Some("admin@hnu.edu.cn".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_last_login() -> TestResult<()> {
        let last_login = personal_data_single(include_str!("test_data/id_data/lastlogin.json"))?;
        assert_eq!(last_login._name, "最近一次登录时间");
        assert_eq!(last_login.value, "2077-06-15 16:04:00");
        assert_eq!(last_login.unit, None);
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_lib() -> TestResult<()> {
        let lib = personal_data_single(include_str!("test_data/id_data/lib.json"))?;
        assert_eq!(lib._name, "待还图书");
        assert_eq!(lib.value, "0");
        assert_eq!(lib.unit, Some("".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_net() -> TestResult<()> {
        let net = personal_data_single(include_str!("test_data/id_data/net.json"))?;
        assert_eq!(net._name, "流量查询");
        assert_eq!(net.value, "114514");
        assert_eq!(net.unit, Some("G".to_string()));

        Ok(())
    }

    #[test]
    fn test_net_convert_to_byte() -> TestResult<()> {
        assert_eq!(
            net_convert_to_byte(1.0, Some("GB".to_string())),
            1024 * 1024 * 1024
        );
        assert_eq!(
            net_convert_to_byte(1.0, Some("MB".to_string())),
            1024 * 1024
        );
        assert_eq!(net_convert_to_byte(1.0, Some("KB".to_string())), 1024);
        assert_eq!(net_convert_to_byte(1.0, Some("B".to_string())), 1);
        assert_eq!(net_convert_to_byte(1.0, None), 1);

        Ok(())
    }

    #[test]
    fn test_parse_personal_data() -> TestResult<()> {
        let summary = personal_data([
            (
                "book.bookNum".to_string(),
                include_str!("test_data/id_data/lib.json").to_string(),
            ),
            (
                "mail.unread".to_string(),
                include_str!("test_data/id_data/email.json").to_string(),
            ),
            (
                "card.balance".to_string(),
                include_str!("test_data/id_data/card.json").to_string(),
            ),
            (
                "statistic.lastLoginTime".to_string(),
                include_str!("test_data/id_data/lastlogin.json").to_string(),
            ),
            (
                "net.used".to_string(),
                include_str!("test_data/id_data/net.json").to_string(),
            ),
        ])?;

        assert_eq!(summary.lib_borrow, Some(0));
        assert_eq!(summary.email.as_deref(), Some("admin@hnu.edu.cn"));
        assert_eq!(summary.mail_unread, Some(1));
        assert_eq!(summary.balance, Some(81.81));
        assert_eq!(
            summary.last_login_time,
            Some(NaiveDateTime::parse_from_str(
                "2077-06-15 16:04:00",
                "%Y-%m-%d %H:%M:%S",
            )?)
        );
        assert_eq!(summary.net_used, Some(114514 * 1024 * 1024 * 1024));

        Ok(())
    }
}
