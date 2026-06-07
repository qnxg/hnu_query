use std::convert::Infallible;

use crate::netflow::pay_info::raw::RawPayInfo;

pub fn overdue_payment(raw_data: RawPayInfo) -> Result<f64, crate::Error<Infallible>> {
    Ok(raw_data.Total)
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_extract_overdue_payment() -> TestResult<()> {
        let raw_data: RawPayInfo = serde_json::from_str(include_str!("test_data/getpayinfo.json"))?;
        let result = overdue_payment(raw_data)?;
        assert_eq!(result, 0.0);

        Ok(())
    }
}
