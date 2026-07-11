use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    pt::login::PtToken,
    utils::client,
};
use std::convert::Infallible;

const UNREAD_EMAIL_URL: &str = "https://pt.hnu.edu.cn/api/v1/email/unRead/count";

pub async fn unread_email_count(pt_token: &PtToken) -> Result<String, crate::Error<Infallible>> {
    client
        .get(UNREAD_EMAIL_URL)
        .headers(pt_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}
