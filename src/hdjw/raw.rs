use crate::{
    error::{MapUnexpectedErr, parse_err},
    hdjw::error::TokenExpired,
};
use reqwest::Response;
use serde_json::Value;

pub trait HdjwResponseExtractor {
    async fn extract_data(self) -> Result<Value, crate::Error<TokenExpired>>;
}

impl HdjwResponseExtractor for Response {
    /// 提取教务系统响应中的数据，解析到 json 格式
    ///
    /// 特判了课程分数详情的响应，这种响应是 html 格式，这里包装成 Value::String 返回
    async fn extract_data(self) -> Result<Value, crate::Error<TokenExpired>> {
        let body = self.text().await.unexpected_err()?;
        // 特判课程分数详情的响应
        if body.contains("window.initQzTable") {
            return Ok(Value::String(body));
        }
        let json = match serde_json::from_str::<Value>(&body) {
            Ok(json) => json,
            Err(_) => return Err(parse_err(&body)),
        };
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
}
