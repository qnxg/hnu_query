mod raw;

use crate::netflow::detail::raw::Detail as RawDetail;
use crate::netflow::detail::raw::raw_day_detail_data;
use crate::netflow::detail::raw::raw_month_detail_data;
use crate::netflow::login::NetflowToken;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// 校园网流量明细
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Detail {
    /// 总流量
    ///
    /// 单位: KB
    pub total: f64,
    /// 上传流量
    ///
    /// 单位: KB
    pub upload: f64,
    /// 下载流量
    ///
    /// 单位: KB
    pub download: f64,
    /// 明细，包含了具体是哪些应用耗费了多少流量
    pub items: Vec<DetailItem>,
}

/// 校园网流量明细项
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DetailItem {
    /// 应用名称
    ///
    /// 按 `/` 将分类分割，比如 `/基础协议/SSL`、`/即时消息/QQ/音视频`
    pub app: String,
    /// 总流量
    ///
    /// 单位: KB
    pub total: f64,
    /// 下载流量
    ///
    /// 单位: KB
    pub download: f64,
    /// 上传流量
    ///
    /// 单位: KB
    pub upload: f64,
    /// 所占比例
    ///
    /// 位于 [0, 1] 之间的小数
    pub percentage: f64,
}

fn convert(raw_data: RawDetail) -> Detail {
    Detail {
        total: raw_data.AllTotal,
        upload: raw_data.AllUpload,
        download: raw_data.AllDownload,
        items: raw_data
            .FloatDetailList
            .into_iter()
            .map(|item| DetailItem {
                app: item.App,
                total: item.Total,
                download: item.Download,
                upload: item.Upload,
                percentage: item.Per,
            })
            .collect::<Vec<DetailItem>>(),
    }
}

/// 获取月流量明细
///
/// # Arguments
///
/// - `network_token`: 校园网令牌，可以通过 [NetflowToken::acquire_by_cas_login] 获取
/// - `year`: 年份
/// - `month`: 月份
///
/// # Returns
///
/// 返回一个包含月流量明细的 [Detail] 实例
pub async fn get_month_detail(
    network_token: &NetflowToken,
    year: u16,
    month: u8,
) -> Result<Detail, crate::Error<Infallible>> {
    let res = raw_month_detail_data(network_token, year, month)
        .await
        .map(convert)?;
    Ok(res)
}

/// 获取日流量明细
///
/// # Arguments
///
/// - `network_token`: 校园网令牌，可以通过 [NetflowToken::acquire_by_cas_login] 获取
/// - `year`: 年份
/// - `month`: 月份
/// - `day`: 日期
///
/// # Returns
///
/// 返回一个包含日流量明细的 [Detail] 实例
pub async fn get_day_detail(
    network_token: &NetflowToken,
    year: u16,
    month: u8,
    day: u8,
) -> Result<Detail, crate::Error<Infallible>> {
    let res = raw_day_detail_data(network_token, year, month, day)
        .await
        .map(convert)?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        netflow::test::get_netflow_token,
        test::{TEST_DAY, TEST_MONTH, TEST_YEAR},
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_month_detail() {
        let token = get_netflow_token().await.unwrap();
        let res = get_month_detail(&token, *TEST_YEAR, *TEST_MONTH)
            .await
            .unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_day_detail() {
        let token = get_netflow_token().await.unwrap();
        let day_detail = get_day_detail(&token, *TEST_YEAR, *TEST_MONTH, *TEST_DAY)
            .await
            .unwrap();
        println!("{:#?}", day_detail);
    }
}
