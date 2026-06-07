use crate::pt::email::raw::RawUnreadEmail;
use std::convert::Infallible;

pub fn email_unread_count(
    raw_data: RawUnreadEmail,
) -> Result<Option<u32>, crate::Error<Infallible>> {
    Ok(raw_data.unReadCount)
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_email_unread_count_success() -> TestResult<()> {
        let raw_data: RawUnreadEmail =
            serde_json::from_str(include_str!("test_data/email_unread_success.json"))?;
        let count = email_unread_count(raw_data)?;

        assert_eq!(count, Some(6));

        Ok(())
    }

    #[test]
    fn test_email_unread_count_fail() -> TestResult<()> {
        let raw_data: RawUnreadEmail =
            serde_json::from_str(include_str!("test_data/email_unread_fail.json"))?;
        let count = email_unread_count(raw_data)?;

        assert_eq!(count, None);

        Ok(())
    }
}
