use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    lab::login::LabToken,
    utils::client,
};
use std::convert::Infallible;

const SEM_INFO_URL: &str = "http://10.62.106.112/Common/Common/GetSemDropDownList?HasNull=0";

pub async fn semester(lab_token: &LabToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(SEM_INFO_URL)
        .headers(lab_token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
