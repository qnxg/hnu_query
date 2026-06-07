use crate::error::MapParseErr;
use chrono::NaiveDateTime;
use std::convert::Infallible;

use super::OrderItem;
use crate::netflow::order::raw::RawOrderItem;

pub fn orders(raw_data: Vec<RawOrderItem>) -> Result<Vec<OrderItem>, crate::Error<Infallible>> {
    raw_data
        .into_iter()
        .map(|item| {
            Ok(OrderItem {
                time: item.Month,
                // 考虑到月流量应当不会超过约 8192TB，此处直接转换不会丢失精度
                download_usage: item.Download.unwrap_or_default() as usize,
                upload_usage: item.Upload.unwrap_or_default() as usize,
                over_usage: item.RealOverTraffic,
                should_pay: item.ShouldPay,
                update_time: NaiveDateTime::parse_from_str(&item.UpdateTime, "%Y-%m-%d %H:%M:%S")
                    .parse_err(&item.UpdateTime)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_parse_order() -> TestResult<()> {
        let raw_data: Vec<RawOrderItem> =
            serde_json::from_str(include_str!("test_data/getpagedlist.json"))?;
        let orders = orders(raw_data)?;

        assert_eq!(orders.len(), 3);
        // 仅测试第一项通过即可
        let first_item = &orders[0];
        assert_eq!(first_item.time, "2026-05");
        assert_eq!(first_item.download_usage, 1616764475);
        assert_eq!(first_item.upload_usage, 169202003);
        assert_eq!(first_item.over_usage, 0.0);
        assert_eq!(first_item.should_pay, 0.0);
        assert_eq!(
            first_item.update_time,
            NaiveDate::from_ymd_opt(2026, 6, 1)
                .expect("this should not panic")
                .and_hms_opt(0, 56, 54)
                .expect("this should not panic")
        );

        Ok(())
    }
}
