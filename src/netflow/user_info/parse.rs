use crate::error::{MapParseErr, parse_err};
use serde_json::Value;
use std::convert::Infallible;

use super::UnlockStatus;

/// - `json_str`: 由 [super::fetch::user_info] 返回的数据
pub fn unlock_status(json_str: &str) -> Result<UnlockStatus, crate::Error<Infallible>> {
    let is_locked = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .and_then(|d| d.get("IsLocked"))
        .and_then(|v| v.as_i64())
        .ok_or(parse_err(json_str))?;
    match is_locked {
        0 => Ok(UnlockStatus::Unlocked),
        1 => Ok(UnlockStatus::Locked),
        _ => Ok(UnlockStatus::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_extract_unlock_status() -> TestResult<()> {
        let result = unlock_status(include_str!("test_data/getuserinfo.json"))?;
        assert_eq!(result, UnlockStatus::Unlocked);

        Ok(())
    }
}
