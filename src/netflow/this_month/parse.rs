use std::convert::Infallible;

use super::ThisMonthInfo;
use crate::netflow::this_month::raw::RawThisMonthInfo;

fn try_add_gb_suffix(s: &mut String) {
    if !s.ends_with("GB") {
        *s += "GB";
    }
}

pub fn this_month(raw_data: RawThisMonthInfo) -> Result<ThisMonthInfo, crate::Error<Infallible>> {
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
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_extract_this_month() -> TestResult<()> {
        let raw_data: RawThisMonthInfo =
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
