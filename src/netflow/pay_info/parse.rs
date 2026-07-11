use crate::error::{MapParseErr, parse_err};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawPayInfo {
    Total: f64,
}

/// `json_str` 为 [super::fetch::pay_info] 的返回数据
pub fn overdue_payment(json_str: &str) -> Result<f64, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .map(|v| serde_json::from_value::<RawPayInfo>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;
    Ok(raw_data.Total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_extract_overdue_payment() -> TestResult<()> {
        let result = overdue_payment(include_str!("test_data/getpayinfo.json"))?;
        assert_eq!(result, 0.0);

        Ok(())
    }
}
