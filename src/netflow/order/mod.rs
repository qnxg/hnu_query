mod raw;

use crate::{
    error::MapParseErr,
    netflow::{login::NetflowToken, order::raw::raw_order_data},
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
    pub download_usage: f64,
    /// 使用的上传流量
    ///
    /// 单位: 字节
    pub upload_usage: f64,
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
pub async fn get_order(
    netflow_token: &NetflowToken,
) -> Result<Vec<OrderItem>, crate::Error<Infallible>> {
    let raw_data = raw_order_data(netflow_token).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let temp = OrderItem {
            time: item.Month,
            download_usage: item.Download.unwrap_or_default(),
            upload_usage: item.Upload.unwrap_or_default(),
            over_usage: item.RealOverTraffic,
            should_pay: item.ShouldPay,
            update_time: NaiveDateTime::parse_from_str(&item.UpdateTime, "%Y-%m-%d %H:%M:%S")
                .parse_err(&item.UpdateTime)?,
        };
        res.push(temp);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netflow::test::get_netflow_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_order() {
        let token = get_netflow_token().await.unwrap();
        let order = get_order(&token).await.unwrap();
        println!("{:#?}", order);
    }
}
