use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::client,
};
use serde_json::Value;
use std::convert::Infallible;

const QUERY_URL: &str = "http://wxpay.hnu.edu.cn/api/appElectricCharge/checkRoomNo";

pub async fn raw_electricity_data(
    park: u8,
    building: &str,
    room: &str,
) -> Result<String, crate::Error<Infallible>> {
    let json_str = client
        .get(format!(
            "{}?parkNo={}&buildingNo={}&rechargeType=2&roomNo={}",
            QUERY_URL, park, building, room
        ))
        .header("referer", "http://wxpay.hnu.edu.cn/electricCharge/home/")
        .header("X-Requested-With", "XMLHttpRequest")
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let json = serde_json::from_str::<Value>(&json_str).parse_err(&json_str)?;
    Ok(json
        .get("data")
        .and_then(|data| data.get("Balance"))
        .and_then(|balance| balance.as_str())
        .ok_or(parse_err(&json_str))?
        .to_string())
}
