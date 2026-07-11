use super::{Detail, DetailItem};
use crate::error::{MapParseErr, parse_err};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawDetail {
    AllDownload: f64,
    AllTotal: f64,
    AllUpload: f64,
    FloatDetailList: Vec<RawDetailItem>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawDetailItem {
    App: String,
    Download: f64,
    Per: f64,
    Total: f64,
    Upload: f64,
}

/// `json_str` 为 [super::fetch::detail_by_month] 或 [super::fetch::detail_by_day] 的返回数据
pub fn detail(json_str: &str) -> Result<Detail, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .map(|v| serde_json::from_value::<RawDetail>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;
    let res = Detail {
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
            .collect(),
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_convert_detail() -> TestResult<()> {
        let detail = detail(include_str!("test_data/getfloatdetailbymonth.json"))?;

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
