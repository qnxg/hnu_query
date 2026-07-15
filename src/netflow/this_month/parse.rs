use super::ThisMonthInfo;
use crate::error::{MapParseErr, parse_err};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawThisMonthInfo {
    allBasePackageAmount: f64,
    allExtendPackageAmount: f64,
    allTraffic: String,
    basePackageUsed: f64,
    basePackageUsedPer: f64,
    downloadTraffic: String,
    extendPackageUsed: f64,
    extendPackageUsedPer: f64,
    surplusBasePackage: f64,
    surplusExtendPackage: f64,
    uploadTraffic: String,
}

fn try_add_gb_suffix(s: &mut String) {
    if !s.ends_with("GB") {
        *s += "GB";
    }
}

/// `json_str` 为 [super::fetch::this_month_info] 的返回数据
pub fn this_month_info(json_str: &str) -> Result<ThisMonthInfo, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .map(|v| serde_json::from_value::<RawThisMonthInfo>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err("无法解析本月网络流量信息", json_str))?;
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
    use crate::test::TestResult;

    #[test]
    fn test_extract_this_month() -> TestResult<()> {
        let info = this_month_info(include_str!("test_data/gettrafficinfobythismonth.json"))?;

        assert_eq!(info.upload_usage, "0.14GB".to_string());
        assert_eq!(info.download_usage, "1.67GB".to_string());
        assert_eq!(info.total_usage, "1.81GB".to_string());
        assert_eq!(info.base_package_amount, 40.0);
        assert_eq!(info.base_package_usage, 1.67);
        assert_eq!(info.base_package_usage_percentage, 0.04);
        assert_eq!(info.base_package_surplus, 38.33);
        assert_eq!(info.extend_package_amount, 0.0);
        assert_eq!(info.extend_package_usage, 0.0);
        assert_eq!(info.extend_package_usage_percentage, 0.0);
        assert_eq!(info.extend_package_surplus, 20.0);

        Ok(())
    }
}
