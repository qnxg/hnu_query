mod fetch;
mod parse;

use crate::netflow::login::NetflowToken;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 校园网流量锁定状态
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Copy)]
pub enum UnlockStatus {
    /// 已锁定
    Locked,
    /// 未锁定
    Unlocked,
    /// 未知
    Unknown,
}

/// 获取校园网流量锁定状态
///
/// # Arguments
///
/// - `netflow_token`: 校园网令牌，可以通过 [NetflowToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 返回校园网流量锁定状态
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(netflow_token), fields(subsystem = "netflow",), err)
)]
pub async fn get_unlock_status(
    netflow_token: &NetflowToken,
) -> Result<UnlockStatus, crate::Error<Infallible>> {
    let json_str = fetch::user_info(netflow_token).await?;
    parse::unlock_status(&json_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{netflow::test::get_netflow_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_unlock_status() -> TestResult<()> {
        let token = get_netflow_token().await?;
        let unlock_status = get_unlock_status(&token).await?;
        println!("{:#?}", unlock_status);
        Ok(())
    }
}
