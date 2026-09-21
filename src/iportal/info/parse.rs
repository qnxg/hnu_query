use crate::{
    error::{MapParseErr, parse_err},
    iportal::{
        error::IPortalTokenExpired,
        info::{AccountInfo, Gender},
        util::iportal_jsondata_precheck,
    },
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawAccountInfo {
    uid: String,
    name: String,
    xgh: String,
    identity: String,
    identity_id: String,
    sex: u8,
    depart: String,
    avatar: String,
}

/// 解析当前登录账号信息响应
///
/// # Arguments
///
/// - `json_str`: [`super::fetch::info`] 返回的数据
pub fn account_info(json_str: &str) -> Result<AccountInfo, crate::Error<IPortalTokenExpired>> {
    let data = iportal_jsondata_precheck(json_str)?;
    let info = data
        .get("info")
        .ok_or_else(|| parse_err("无法解析 info 字段", json_str))?
        .clone();
    let raw: RawAccountInfo = serde_json::from_value(info).parse_err(json_str)?;
    Ok(AccountInfo {
        uid: raw.uid,
        name: raw.name,
        id: raw.xgh,
        identity: raw.identity,
        identity_id: raw.identity_id.parse::<u64>().parse_err(json_str)?,
        gender: match raw.sex {
            1 => Gender::Male,
            2 => Gender::Female,
            _ => return Err(parse_err("未知性别代码", json_str)),
        },
        depart: raw.depart,
        avatar: raw.avatar,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_account_info() -> TestResult<()> {
        let info = account_info(include_str!("test_data/info.json"))?;

        assert_eq!(info.uid, "114514");
        assert_eq!(info.name, "电棍");
        assert_eq!(info.id, "191908100721");
        assert_eq!(info.identity, "本科生");
        assert_eq!(info.identity_id, 2002);
        assert_eq!(info.gender, Gender::Male);
        assert_eq!(info.depart, "神秘学院");
        assert_eq!(info.avatar, "http://filtered");

        Ok(())
    }

    #[test]
    fn test_account_info_invalid_shape() -> TestResult<()> {
        let result = account_info(include_str!("test_data/info_invalid_shape.json"));

        assert!(result.is_err());
        Ok(())
    }
}
