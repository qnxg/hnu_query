use serde::Deserialize;

use crate::{
    error::parse_err,
    iportal::{
        error::IPortalTokenExpired,
        info::{AccountInfo, Gender},
        util::iportal_jsondata_precheck,
    },
};

/// iportal 返回的原始账号信息。字段格式和命名以接口响应为准，不能直接作为公共模型使用。
#[derive(Debug, Deserialize)]
#[expect(unused)]
struct RawAccountInfo {
    uid: String,
    name: String,
    xgh: String,
    identity: String,
    identity_id: String,
    sex: u8,
    depart: String,
    // 始终为空字符串，可能是预留字段
    mobile: String,
    // 始终为空字符串，可能是预留字段
    email: String,
    avatar: String,
    // 当前时间，格式为 `yyyy-MM-dd HH:mm:ss`，不携带时区信息
    time: String,
    // 以下字段作用未知
    is_manager: bool,
    is_app_manager: bool,
    is_process_manager: bool,
}

pub fn parse_account_info(
    json_str: &str,
) -> Result<AccountInfo, crate::Error<IPortalTokenExpired>> {
    let data = iportal_jsondata_precheck(json_str)?;
    let info = data
        .get("info")
        .ok_or_else(|| parse_err("无法解析 info 字段", json_str))?
        .clone();
    let raw: RawAccountInfo = serde_json::from_value(info)
        .map_err(|e| parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
    Ok(AccountInfo {
        uid: raw.uid,
        name: raw.name,
        id: raw.xgh,
        identity: raw.identity,
        identity_id: raw.identity_id.parse::<u64>().map_err(|e| {
            parse_err(
                format!("无法解析身份 ID {} 为数字: {}", raw.identity_id, e).as_str(),
                json_str,
            )
        })?,
        gender: match raw.sex {
            1 => Gender::Male,
            2 => Gender::Female,
            _ => Gender::Other(raw.sex),
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
        let info = parse_account_info(include_str!("test_data/info.json"))?;

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
}
