use crate::error::{MapParseErr, parse_err};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

use super::ThisMonthInfo;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawThisMonthInfo {
    pub allBasePackageAmount: f64,
    pub allExtendPackageAmount: f64,
    pub allTraffic: String,
    pub basePackageUsed: f64,
    pub basePackageUsedPer: f64,
    pub downloadTraffic: String,
    pub extendPackageUsed: f64,
    pub extendPackageUsedPer: f64,
    pub surplusBasePackage: f64,
    pub surplusExtendPackage: f64,
    pub uploadTraffic: String,
}

fn try_add_gb_suffix(s: &mut String) {
    if !s.ends_with("GB") {
        *s += "GB";
    }
}

/// 将 [`super::raw::get_traffic_info_by_this_month`] 的返回数据解析为 [`ThisMonthInfo`]
pub fn this_month(raw_data: Value) -> Result<ThisMonthInfo, crate::Error<Infallible>> {
    let raw_converted = raw_data
        .get("data")
        .map(|v| serde_json::from_value::<RawThisMonthInfo>(v.clone()).parse_err(&v.to_string()))
        .transpose()?
        .ok_or(parse_err(&raw_data.to_string()))?;

    let mut res = ThisMonthInfo {
        total_usage: raw_converted.allTraffic,
        upload_usage: raw_converted.uploadTraffic,
        download_usage: raw_converted.downloadTraffic,
        base_package_amount: raw_converted.allBasePackageAmount,
        base_package_usage: raw_converted.basePackageUsed,
        base_package_usage_percentage: raw_converted.basePackageUsedPer,
        base_package_surplus: raw_converted.surplusBasePackage,
        extend_package_amount: raw_converted.allExtendPackageAmount,
        extend_package_usage: raw_converted.extendPackageUsed,
        extend_package_usage_percentage: raw_converted.extendPackageUsedPer,
        extend_package_surplus: raw_converted.surplusExtendPackage,
    };
    try_add_gb_suffix(&mut res.total_usage);
    try_add_gb_suffix(&mut res.upload_usage);
    try_add_gb_suffix(&mut res.download_usage);

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_extract_this_month() -> TestResult<()> {
        let raw_data: Value =
            serde_json::from_str(include_str!("test_data/gettrafficinfobythismonth.json"))?;

        let info = this_month(raw_data)?;

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
