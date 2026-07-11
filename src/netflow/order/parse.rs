use super::OrderItem;
use crate::error::{MapParseErr, parse_err};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawOrderItem {
    Download: Option<f64>,
    Month: String,
    RealOverTraffic: f64,
    ShouldPay: f64,
    UpdateTime: String,
    Upload: Option<f64>,
}

pub fn order(json_str: &str) -> Result<Vec<OrderItem>, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .map(|v| serde_json::from_value::<Vec<RawOrderItem>>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;
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
    use super::*;
    use crate::test::TestResult;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_order() -> TestResult<()> {
        let orders = order(include_str!("test_data/getpagedlist.json"))?;

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
