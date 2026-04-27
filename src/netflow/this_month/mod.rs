mod raw;
mod utils;

use crate::netflow::{
    login::NetflowToken,
    this_month::{raw::raw_this_month_data, utils::try_add_gb_suffix},
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 本月校园网使用信息
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ThisMonthInfo {
    /// 总使用流量
    ///
    /// 带单位，如 `7.54GB`，存在为 `小于0.01GB` 的情况
    pub total_usage: String,
    /// 已经使用的上传流量
    ///
    /// 带单位，如 `7.54GB`，存在为 `小于0.01GB` 的情况
    pub upload_usage: String,
    /// 已经使用的下载流量
    ///
    /// 带单位，如 `7.54GB`，存在为 `小于0.01GB` 的情况
    pub download_usage: String,
    /// 免费流量总量
    ///
    /// 单位：GB
    pub base_package_amount: f64,
    /// 已经使用的免费流量
    ///
    /// 单位：GB
    pub base_package_usage: f64,
    /// 免费流量使用率
    ///
    /// 是一个位于 [0, 1] 之间的浮点数，如 `0.14`
    pub base_package_usage_percentage: f64,
    /// 剩余免费流量
    ///
    /// 单位：GB
    pub base_package_surplus: f64,
    /// 超出流量总量
    ///
    /// 单位：GB
    ///
    /// 没有超出流量时，为 0
    pub extend_package_amount: f64,
    /// 已经使用的超出流量
    ///
    /// 单位：GB
    pub extend_package_usage: f64,
    /// 超出流量使用率
    ///
    /// 是一个位于 [0, 1] 之间的浮点数，如 `0.14`
    pub extend_package_usage_percentage: f64,
    /// 剩余超出流量
    ///
    /// 单位：GB
    ///
    /// 比较奇怪的是，当没有超出流量时，这个值不为零，比如为 `20`
    pub extend_package_surplus: f64,
}

/// 获取本月校园网使用信息
///
/// # Arguments
///
/// - `netflow_token`: 校园网令牌，可以通过 [NetflowToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 本月校园网流量使用信息
pub async fn get_this_month_info(
    netflow_token: &NetflowToken,
) -> Result<ThisMonthInfo, crate::Error<Infallible>> {
    let raw_data = raw_this_month_data(netflow_token).await?;
    let mut res = ThisMonthInfo {
        total_usage: raw_data.allTraffic,
        upload_usage: raw_data.uploadTraffic,
        download_usage: raw_data.downloadTraffic,
        base_package_amount: raw_data.allBasePackageAmount,
        base_package_usage: raw_data.basePackageUsed,
        base_package_usage_percentage: raw_data.basePackageUsedPer,
        base_package_surplus: raw_data.surplusBasePackage,
        extend_package_amount: raw_data.allExtendPackageAmount,
        extend_package_usage: raw_data.extendPackageUsed,
        extend_package_usage_percentage: raw_data.extendPackageUsedPer,
        extend_package_surplus: raw_data.surplusExtendPackage,
    };
    try_add_gb_suffix(&mut res.total_usage);
    try_add_gb_suffix(&mut res.upload_usage);
    try_add_gb_suffix(&mut res.download_usage);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netflow::test::get_netflow_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_this_month_info() {
        let token = get_netflow_token().await.unwrap();
        let this_month_info = get_this_month_info(&token).await.unwrap();
        println!("{:#?}", this_month_info);
    }
}
