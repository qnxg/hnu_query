use crate::{
    error::{MapParseErr, parse_err},
    gym::error::TokenExpired,
};
use serde_json::Value;

/// 将响应字符串解析成 Value 并检查：
///
/// - 响应是否表明 cookie 过期，如果是的话返回 [TokenExpired] 错误
/// - 响应的 status 字段是否为 1
///
/// # Returns
///
/// 如果检查通过，则将响应的 data 字段返回
pub fn gym_response(json_str: &str) -> Result<Value, crate::Error<TokenExpired>> {
    let json: Value = serde_json::from_str(json_str).parse_err(json_str)?;
    // 典型的异常response body：
    // {"data":[],"info":"登录失效","status":-1}
    if json
        .get("info")
        .and_then(|v| v.as_str())
        .ok_or_else(|| parse_err(json_str))?
        .contains("登录失效")
    {
        return Err(crate::Error::Other(TokenExpired));
    }
    if json.get("status").and_then(|v| v.as_i64()) != Some(1) {
        return Err(parse_err(json_str));
    }
    let Some(data) = json.get("data") else {
        return Err(parse_err(json_str));
    };
    Ok(data.clone())
}
