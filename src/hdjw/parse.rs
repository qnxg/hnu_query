use crate::{error::MapParseErr, hdjw::error::TokenExpired};
use serde_json::Value;

/// 解析教务系统响应，同时会检查 cookie 是否过期，如果过期则返回 [TokenExpired] 错误
///
/// 特判了课程分数详情的响应，这种响应是 html 格式，这里包装成 Value::String 返回
pub fn hdjw_response(body_str: &str) -> Result<Value, crate::Error<TokenExpired>> {
    // 特判课程分数详情的响应
    if body_str.contains("window.initQzTable") {
        return Ok(Value::String(body_str.to_string()));
    }
    let json = serde_json::from_str::<Value>(body_str).parse_err(body_str)?;
    // 典型的 cookie 失效时的 response body：
    // {"flag1":2,"msgContent":"è¯·å…ˆç™»å½•ç³»ç»Ÿ"}
    // 这里只判断 flag1 字段，因为 msgContent 是乱码，不好说
    if let Some(Value::Number(flag1)) = json.get("flag1")
        && flag1.as_i64() == Some(2)
    {
        return Err(crate::Error::Other(TokenExpired));
    }
    Ok(json)
}
