mod fetch;
mod parse;

use crate::netflow::login::NetflowToken;
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
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(netflow_token), fields(subsystem = "netflow",), err)
)]
pub async fn get_overdue_payment(
    netflow_token: &NetflowToken,
) -> Result<f64, crate::Error<Infallible>> {
    let json_str = fetch::pay_info(netflow_token).await?;
    parse::overdue_payment(&json_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{netflow::test::get_netflow_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_overdue_payment() -> TestResult<()> {
        let token = get_netflow_token().await?;
        let overdue_payment = get_overdue_payment(&token).await?;
        println!("{:#?}", overdue_payment);
        Ok(())
    }
}
