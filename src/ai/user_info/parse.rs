use crate::error::{MapParseErr, parse_err};
use serde_json::Value;
use std::convert::Infallible;

/// `json_str` 可接收来自 [super::fetch::user_info_data] 的返回值
pub fn remaining_quota(json_str: &str) -> Result<usize, crate::Error<Infallible>> {
    let res: Value = serde_json::from_str(json_str).parse_err(json_str)?;
    let total_granted = res["data"]["total_granted"]
        .as_u64()
        .ok_or_else(|| parse_err(json_str))? as usize;
    Ok(total_granted)
}
