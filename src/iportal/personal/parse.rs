use crate::{
    error::parse_err,
    iportal::{
        error::IPortalTokenExpired,
        personal::{PersonalDataItem, PersonalDataTypeEnum},
        util::iportal_jsondata_precheck,
    },
};

pub fn parse_personal_data_query_ids(
    json_str: &str,
) -> Result<Vec<PersonalDataTypeEnum>, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str);
    let arr = json_value?
        .get("data")
        .ok_or(parse_err("JSON解析错误", json_str))?
        .as_array()
        .ok_or(parse_err("JSON解析错误", json_str))?
        .clone();
    let mut items = Vec::new();
    for item in arr {
        let data_item: PersonalDataTypeEnum = serde_json::from_value(item.clone())
            .map_err(|e| parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
        items.push(data_item);
    }
    Ok(items)
}

pub fn parse_personal_data(
    json_str: &str,
) -> Result<PersonalDataItem, crate::Error<IPortalTokenExpired>> {
    let json_value: Result<serde_json::Value, crate::Error<IPortalTokenExpired>> =
        iportal_jsondata_precheck(json_str);
    let data_item = json_value?
        .get("data")
        .ok_or(parse_err("JSON解析错误", json_str))?
        .clone();
    let item: PersonalDataItem = serde_json::from_value(data_item)
        .map_err(|e| parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_personal_data_query_ids() -> TestResult<()> {
        let items = parse_personal_data_query_ids(include_str!("test_data/ids.json"))?;

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
        let card = parse_personal_data(include_str!("test_data/id_data/card.json"))?;
        assert_eq!(card.name, "一卡通余额");
        assert_eq!(card.value, "81.81");
        assert_eq!(card.unit, Some("元".to_string()));
        assert_eq!(card.email, None);
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_email() -> TestResult<()> {
        let email = parse_personal_data(include_str!("test_data/id_data/email.json"))?;
        assert_eq!(email.name, "未读邮件");
        assert_eq!(email.value, "1");
        assert_eq!(email.unit, None);
        assert_eq!(email.email, Some("admin@hnu.edu.cn".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_last_login() -> TestResult<()> {
        let last_login = parse_personal_data(include_str!("test_data/id_data/lastlogin.json"))?;
        assert_eq!(last_login.name, "最近一次登录时间");
        assert_eq!(last_login.value, "2077-06-15 16:04:00");
        assert_eq!(last_login.unit, None);
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_lib() -> TestResult<()> {
        let lib = parse_personal_data(include_str!("test_data/id_data/lib.json"))?;
        assert_eq!(lib.name, "待还图书");
        assert_eq!(lib.value, "0");
        assert_eq!(lib.unit, Some("".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_personal_data_net() -> TestResult<()> {
        let net = parse_personal_data(include_str!("test_data/id_data/net.json"))?;
        assert_eq!(net.name, "流量查询");
        assert_eq!(net.value, "114514");
        assert_eq!(net.unit, Some("G".to_string()));

        Ok(())
    }
}
