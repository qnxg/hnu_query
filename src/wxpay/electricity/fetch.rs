use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
};
use std::convert::Infallible;

const QUERY_URL: &str = "http://wxpay.hnu.edu.cn/api/appElectricCharge/checkRoomNo";

pub async fn electricity(
    park: u8,
    building: &str,
    room: &str,
) -> Result<String, crate::Error<Infallible>> {
    client
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
        .unexpected_err()
}
