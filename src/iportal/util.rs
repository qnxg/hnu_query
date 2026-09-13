use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr, parse_err},
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
};
use reqwest::{Response, header::COOKIE};
use reqwest_middleware::RequestBuilder;

pub trait IPortalRequestBuilderExt {
    async fn send_with_token(
        self,
        token: &IPortalToken,
    ) -> Result<Response, crate::Error<IPortalTokenExpired>>;
}

impl IPortalRequestBuilderExt for RequestBuilder {
    async fn send_with_token(
        self,
        token: &IPortalToken,
    ) -> Result<Response, crate::Error<IPortalTokenExpired>> {
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
) -> Result<serde_json::Value, crate::Error<IPortalTokenExpired>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iportal_jsondata_precheck_error_response() {
        let result = iportal_jsondata_precheck(
            r#"{
                "e": 1,
                "m": "参数错误",
                "d": null
            }"#,
        );

        assert!(result.is_err());
    }
}
