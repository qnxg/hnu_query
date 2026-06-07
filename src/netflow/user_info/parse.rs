use crate::error::parse_err;
use serde_json::Value;
use std::convert::Infallible;

use super::UnlockStatus;

/// # Parameters
///
/// - `raw_data`: 由 [super::raw::get_user_info] 返回的原始数据
pub fn unlock_status(raw_data: Value) -> Result<UnlockStatus, crate::Error<Infallible>> {
    let is_locked = raw_data
        .get("data")
        .and_then(|d| d.get("IsLocked"))
        .and_then(|v| v.as_i64())
        .ok_or(parse_err(&raw_data.to_string()))?;
    match is_locked {
        0 => Ok(UnlockStatus::Unlocked),
        1 => Ok(UnlockStatus::Locked),
        _ => Ok(UnlockStatus::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_extract_unlock_status() -> TestResult<()> {
        let raw_data: Value = serde_json::from_str(include_str!("test_data/getuserinfo.json"))?;
        let result = unlock_status(raw_data)?;
        assert_eq!(result, UnlockStatus::Unlocked);

        Ok(())
    }
}
