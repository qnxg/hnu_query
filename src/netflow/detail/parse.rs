use crate::error::{MapParseErr, parse_err};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

use super::{Detail, DetailItem};

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawDetail {
    pub AllDownload: f64,
    pub AllTotal: f64,
    pub AllUpload: f64,
    pub FloatDetailList: Vec<RawDetailItem>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawDetailItem {
    pub App: String,
    pub Download: f64,
    pub Per: f64,
    pub Total: f64,
    pub Upload: f64,
}

/// 解析 [`super::raw::get_float_detail_by_month`] 或 [`super::raw::get_float_detail_by_day`] 的返回数据为 [`Detail`]
pub fn detail(raw_data: Value) -> Result<Detail, crate::Error<Infallible>> {
    let raw_converted = raw_data
        .get("data")
        .map(|v| serde_json::from_value::<RawDetail>(v.clone()).parse_err(&v.to_string()))
        .transpose()?
        .ok_or(parse_err(&raw_data.to_string()))?;

    Ok(Detail {
        total: raw_converted.AllTotal,
        upload: raw_converted.AllUpload,
        download: raw_converted.AllDownload,
        items: raw_converted
            .FloatDetailList
            .into_iter()
            .map(|item| DetailItem {
                app: item.App,
                total: item.Total,
                download: item.Download,
                upload: item.Upload,
                percentage: item.Per,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_convert_detail() -> TestResult<()> {
        let raw_data: Value =
            serde_json::from_str(include_str!("test_data/getfloatdetailbymonth.json"))?;

        let detail = detail(raw_data)?;

        assert_eq!(detail.download, 1751143.21);
        assert_eq!(detail.upload, 155597061.0);
        assert_eq!(detail.total, 1948767711.0);

        assert_eq!(detail.items.len(), 3);
        // 仅测试第一项通过即可
        let first_item = &detail.items[0];
        assert_eq!(first_item.app, "/网络游戏/steam平台");
        assert_eq!(first_item.total, 703434.19);
        assert_eq!(first_item.download, 678507.29);
        assert_eq!(first_item.upload, 24926.9);
        assert_eq!(first_item.percentage, 0.37);

        Ok(())
    }
}
