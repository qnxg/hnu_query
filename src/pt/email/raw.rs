use std::convert::Infallible;

use serde::Deserialize;
use serde_json::Value;

use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    pt::login::PtToken,
    utils::client,
};

const UNREAD_EMAIL_URL: &str = "https://pt.hnu.edu.cn/api/v1/email/unRead/count";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct UnreadEmail {
    pub unReadCount: Option<u32>,
}

pub async fn raw_unread_email_data(
    pt_token: &PtToken,
) -> Result<UnreadEmail, crate::Error<Infallible>> {
    let headers = pt_token.headers().clone();
    let json_str = client
        .get(UNREAD_EMAIL_URL)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: UnreadEmail = serde_json::from_str::<Value>(&json_str)
        .parse_err(&json_str)?
        .get("data")
        .map(|v| serde_json::from_value(v.clone()).parse_err(&json_str))
        .transpose()?
        .ok_or(parse_err(&json_str))?;
    Ok(res)
}
