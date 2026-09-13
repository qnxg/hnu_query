use crate::{
    cas::error::TokenExpired,
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr, parse_err},
    iportal::login::IPortalToken,
};
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use reqwest::{Response, header::COOKIE};
use reqwest_middleware::RequestBuilder;
use serde::{Deserialize, Deserializer};

pub trait IPortalRequestBuilderExt {
    async fn send_with_token(
        self,
        token: &IPortalToken,
    ) -> Result<Response, crate::Error<TokenExpired>>;
}

impl IPortalRequestBuilderExt for RequestBuilder {
    async fn send_with_token(
        self,
        token: &IPortalToken,
    ) -> Result<Response, crate::Error<TokenExpired>> {
        let cookie = token
            .headers()
            .get(COOKIE)
            .cloned()
            .ok_or_else(|| "缺少 Cookie".to_string())
            .unexpected_err()?;

        self.header(COOKIE, cookie)
            .send()
            .await
            .network_err()?
            .status_code_err()
            .await
    }
}

pub fn iportal_jsondata_precheck(
    json_str: &str,
) -> Result<serde_json::Value, crate::Error<TokenExpired>> {
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
    if json_value.get("e").and_then(|e| e.as_i64()) != Some(0) {
        let msg = json_value
            .get("m")
            .and_then(|m| m.as_str())
            .unwrap_or("未知错误");
        return Err(parse_err(
            format!("参数错误, 服务器返回信息: {}", msg).as_str(),
            json_str,
        ));
    }
    let data = json_value
        .get("d")
        .ok_or(parse_err("JSON解析错误", json_str))?;
    Ok(data.clone())
}

pub fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt =
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)?;
    let timezone = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| serde::de::Error::custom("invalid timezone"))?;
    let dt = timezone
        .from_local_datetime(&dt)
        .single()
        .ok_or_else(|| serde::de::Error::custom("invalid datetime"))?;
    Ok(dt.to_utc())
}
