mod fetch;
mod parse;

use crate::{
    netflow::login::NetflowToken,
    utils::obs::{fetch_time, parse_time, traced},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 校园网流量账单信息
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OrderItem {
    /// 时间
    ///
    /// 为 `YYYY-MM` 格式，如 `2025-01`
    pub time: String,
    /// 使用的下载流量
    ///
    /// 单位: 字节
    pub download_usage: usize,
    /// 使用的上传流量
    ///
    /// 单位: 字节
    pub upload_usage: usize,
    /// 超额流量
    ///
    /// 单位: GB
    pub over_usage: f64,
    /// 应缴费用
    ///
    /// 单位: 元
    pub should_pay: f64,
    /// 更新时间
    pub update_time: NaiveDateTime,
}

/// 获取校园网流量账单信息
///
/// # Arguments
///
/// - `netflow_token`: 校园网令牌，可以通过 [NetflowToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 返回一个包含校园网流量账单信息的列表
#[traced(subsystem = "netflow", skip(netflow_token))]
pub async fn get_order(
    netflow_token: &NetflowToken,
) -> Result<Vec<OrderItem>, crate::Error<Infallible>> {
    let json_str = fetch_time!(fetch::order(netflow_token).await)?;
    parse_time!(parse::order(&json_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{netflow::test::get_netflow_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_order() -> TestResult<()> {
        let token = get_netflow_token().await?;
        let order = get_order(&token).await?;
        println!("{:#?}", order);
        Ok(())
    }
}
