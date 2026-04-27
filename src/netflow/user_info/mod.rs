mod raw;

use crate::{
    error::parse_err,
    netflow::{login::NetflowToken, user_info::raw::raw_user_info_data},
};
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
    let raw_data = raw_user_info_data(netflow_token).await?;
    let is_locked = raw_data
        .get("IsLocked")
        .and_then(|v| v.as_i64())
        .ok_or(parse_err(&raw_data.to_string()))?;
    match is_locked {
        0 => Ok(UnlockStatus::Unlocked),
        1 => Ok(UnlockStatus::Locked),
        _ => Ok(UnlockStatus::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netflow::test::get_netflow_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_unlock_status() {
        let token = get_netflow_token().await.unwrap();
        let unlock_status = get_unlock_status(&token).await.unwrap();
        println!("{:#?}", unlock_status);
    }
}
