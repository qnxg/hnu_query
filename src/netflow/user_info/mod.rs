mod parse;
mod raw;

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
pub async fn get_unlock_status(
    netflow_token: &NetflowToken,
) -> Result<UnlockStatus, crate::Error<Infallible>> {
    let raw_data = raw::get_user_info(netflow_token).await?;
    parse::unlock_status(raw_data)
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
