use crate::{
    error::{MapParseErr, parse_err},
    iportal::error::IPortalTokenExpired,
};

pub(super) fn iportal_jsondata_precheck(
    json_str: &str,
) -> Result<serde_json::Value, crate::Error<IPortalTokenExpired>> {
    let json_value: serde_json::Value = serde_json::from_str(json_str).parse_err(json_str)?;
    let error_code = json_value
        .get("e")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| parse_err("无法解析 e 字段", json_str))?;
    if error_code != 0 {
        let msg = json_value
            .get("m")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("未知错误");
        return Err(parse_err(
            format!("参数错误, 服务器返回信息: {msg}").as_str(),
            json_str,
        ));
    }
    let data = json_value
        .get("d")
        .cloned()
        .ok_or_else(|| parse_err("无法解析数据", json_str))?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iportal_jsondata_precheck_error_response() -> crate::test::TestResult<()> {
        let json_str = include_str!("test_data/error.json");
        let result = iportal_jsondata_precheck(json_str);

        match result {
            Err(crate::Error::Parse(error)) => assert_eq!(error.data(), json_str),
            _ => return Err("错误响应未返回 ParseError".into()),
        }
        Ok(())
    }

    #[test]
    fn test_iportal_jsondata_precheck_malformed_json() -> crate::test::TestResult<()> {
        let json_str = include_str!("test_data/malformed.json");
        let result = iportal_jsondata_precheck(json_str);

        match result {
            Err(crate::Error::Parse(error)) => assert_eq!(error.data(), json_str),
            _ => return Err("格式错误的 JSON 未返回 ParseError".into()),
        }
        Ok(())
    }
}
