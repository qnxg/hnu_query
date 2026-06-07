use std::convert::Infallible;

use super::{Detail, DetailItem};
use crate::netflow::detail::raw::RawDetail;

/// 将 [`super::raw::get_float_detail_by_month`] 或 [`super::raw::get_float_detail_by_day`] 的返回数据转换为 [`Detail`]
pub fn detail(raw_data: RawDetail) -> Result<Detail, crate::Error<Infallible>> {
    Ok(Detail {
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
    })
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;
    use crate::netflow::detail::raw::RawDetail;

    #[test]
    fn test_convert_detail() -> TestResult<()> {
        let raw_data: RawDetail =
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
