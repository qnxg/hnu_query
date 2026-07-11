use crate::{
    ai::token::TokenInfo,
    error::{MapParseErr, parse_err},
};
use serde_json::Value;
use std::convert::Infallible;

/// `json_str` 可接收来自 [super::fetch::token_list] 的返回值
pub fn token_list(json_str: &str) -> Result<Vec<TokenInfo>, crate::Error<Infallible>> {
    let json: Value = serde_json::from_str(json_str).parse_err(json_str)?;
    let tokens: Vec<TokenInfo> = if json["data"].is_null() {
        Vec::new()
    } else {
        serde_json::from_value(json["data"].clone()).parse_err(json_str)?
    };
    Ok(tokens)
}

/// `json_str` 可接收来自 [super::fetch::token_key] 的返回值
pub fn token_key(json_str: &str) -> Result<String, crate::Error<Infallible>> {
    let json: Value = serde_json::from_str(json_str).parse_err(json_str)?;
    let key = json["data"]["key"]
        .as_str()
        .ok_or_else(|| parse_err(json_str))?;
    Ok(key.to_string())
}

/// 检查操作是否成功
///
/// `json_str` 可接收来自 [super::fetch::delete_token] 和 [super::fetch::create_token] 的返回值
pub fn check_action_success(json_str: &str) -> Result<bool, crate::Error<Infallible>> {
    let json: Value = serde_json::from_str(json_str).parse_err(json_str)?;
    let success = json["success"]
        .as_bool()
        .ok_or_else(|| parse_err(json_str))?;
    Ok(success)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_check_action_success() -> TestResult<()> {
        let json_str = include_str!("test_data/apply-token.json");
        let success = check_action_success(json_str)?;
        assert!(success);
        Ok(())
    }

    #[test]
    fn test_token_list() -> TestResult<()> {
        let json_str = include_str!("test_data/tokens.json");
        let tokens = token_list(json_str)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_name, "hello");
        assert_eq!(tokens[0].id, 3296);
        Ok(())
    }

    #[test]
    fn test_token_key() -> TestResult<()> {
        let json_str = include_str!("test_data/key.json");
        let key = token_key(json_str)?;
        assert_eq!(key, "zF0Asilc5eRnGbwafZ5gDzIH");
        Ok(())
    }
}
