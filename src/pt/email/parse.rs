use crate::error::{MapParseErr, parse_err};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawUnreadEmail {
    unReadCount: Option<u32>,
}

/// `json_str` 为 [super::fetch::unread_email_count] 的返回数据
pub fn email_unread_count(json_str: &str) -> Result<Option<u32>, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .map(|v| serde_json::from_value::<RawUnreadEmail>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;
    Ok(raw_data.unReadCount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_email_unread_count_success() -> TestResult<()> {
        let count = email_unread_count(include_str!("test_data/email_unread_success.json"))?;
        assert_eq!(count, Some(6));
        Ok(())
    }

    #[test]
    fn test_email_unread_count_fail() -> TestResult<()> {
        let count = email_unread_count(include_str!("test_data/email_unread_fail.json"))?;
        assert_eq!(count, None);
        Ok(())
    }
}
