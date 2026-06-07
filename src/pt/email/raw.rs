use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    pt::login::PtToken,
    utils::client,
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

const UNREAD_EMAIL_URL: &str = "https://pt.hnu.edu.cn/api/v1/email/unRead/count";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct RawUnreadEmail {
    pub unReadCount: Option<u32>,
}

pub async fn get_email_unread_count(
    pt_token: &PtToken,
) -> Result<RawUnreadEmail, crate::Error<Infallible>> {
    let json_str = client
        .get(UNREAD_EMAIL_URL)
        .headers(pt_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;

    let data = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(data)
}
