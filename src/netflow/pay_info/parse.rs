use crate::error::parse_err;
use serde_json::Value;
use std::convert::Infallible;

/// 从 [`super::raw::get_pay_info`] 中提取欠费金额
pub fn overdue_payment(raw_data: Value) -> Result<f64, crate::Error<Infallible>> {
    raw_data
        .get("data")
        .and_then(|d| d.get("Total"))
        .and_then(|t| t.as_f64())
        .ok_or(parse_err(&raw_data.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_extract_overdue_payment() -> TestResult<()> {
        let raw_data: Value = serde_json::from_str(include_str!("test_data/getpayinfo.json"))?;
        let result = overdue_payment(raw_data)?;
        assert_eq!(result, 0.0);

        Ok(())
    }
}
