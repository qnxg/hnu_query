use crate::{
    error::parse_err,
    iportal::{error::IPortalTokenExpired, info::AccountInfo, util::iportal_jsondata_precheck},
};

pub fn parse_account_info(
    json_str: &str,
) -> Result<AccountInfo, crate::Error<IPortalTokenExpired>> {
    let data = iportal_jsondata_precheck(json_str)?;
    let info = data
        .get("info")
        .ok_or_else(|| parse_err("无法解析 info 字段", json_str))?
        .clone();
    let account_info: AccountInfo = serde_json::from_value(info)
        .map_err(|e| parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
    Ok(account_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_account_info() -> TestResult<()> {
        let info = parse_account_info(include_str!("test_data/info.json"))?;

        assert_eq!(info.uid, "114514");
        assert_eq!(info.name, "电棍");
        assert_eq!(info.xgh, "191908100721");
        assert_eq!(info.identity, "本科生");
        assert_eq!(info.identity_id, "2002");
        assert_eq!(info.sex, 1);
        assert_eq!(info.depart, "神秘学院");
        assert_eq!(info.mobile, "");
        assert_eq!(info.email, "");
        assert_eq!(info.avatar, "http://filtered");
        assert_eq!(
            info.time,
            chrono::NaiveDateTime::parse_from_str("2077-06-15 16:04:00", "%Y-%m-%d %H:%M:%S")?
        );
        assert!(!info.is_manager);
        assert!(!info.is_app_manager);
        assert!(!info.is_process_manager);

        Ok(())
    }
}
