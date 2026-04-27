use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    lab::login::LabToken,
    utils::client,
};
use serde::Deserialize;
use std::convert::Infallible;

const SEM_INFO_URL: &str = "http://10.62.106.112/Common/Common/GetSemDropDownList?HasNull=0";

#[derive(Deserialize, Debug)]
pub struct SemesterItem {
    pub id: String,
    pub text: String,
}

pub async fn raw_semester_data(
    lab_token: &LabToken,
) -> Result<Vec<SemesterItem>, crate::Error<Infallible>> {
    let headers = lab_token.headers().clone();
    let json_str = client
        .get(SEM_INFO_URL)
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let res: Vec<SemesterItem> = serde_json::from_str(&json_str).parse_err(&json_str)?;
    Ok(res)
}
