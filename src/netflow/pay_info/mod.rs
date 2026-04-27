mod raw;

use crate::netflow::{login::NetflowToken, pay_info::raw::raw_pay_info_data};
use std::convert::Infallible;

/// 获取欠费金额
///
/// # Arguments
///
/// - `netflow_token`: 校园网令牌，可以通过 [NetflowToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 欠费金额
pub async fn get_overdue_payment(
    netflow_token: &NetflowToken,
) -> Result<f64, crate::Error<Infallible>> {
    let raw_data = raw_pay_info_data(netflow_token).await?;
    Ok(raw_data.Total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netflow::test::get_netflow_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_overdue_payment() {
        let token = get_netflow_token().await.unwrap();
        let overdue_payment = get_overdue_payment(&token).await.unwrap();
        println!("{:#?}", overdue_payment);
    }
}
