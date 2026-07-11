use super::{Rank, RankDetail};
use crate::{error::parse_err, hdjw::error::TokenExpired};
use serde_json::Value;

/// 湖大的教务系统的字段返回类型难说，此函数用于尝试所有可能的类型
fn parse_number(value: &Value) -> Option<String> {
    value
        .as_f64()
        .map(|f| f.to_string())
        .or(value.as_i64().map(|i| i.to_string()))
        .or(value.as_str().map(|s| s.to_string()))
}

fn rank_detail(value: &Value) -> Result<RankDetail, crate::Error<TokenExpired>> {
    Ok(RankDetail {
        arithmetic: parse_number(&value["avgzcj"]).ok_or(parse_err(&value.to_string()))?,
        arithmetic_rank: value["avgzcjpm"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(parse_err(&value.to_string()))?,
        weighted: parse_number(&value["pjxfj"]).ok_or(parse_err(&value.to_string()))?,
        weighted_rank: value["pjxfjpm"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(parse_err(&value.to_string()))?,
        gpa: parse_number(&value["pjxfjd"]).ok_or(parse_err(&value.to_string()))?,
        gpa_rank: value["pjxfjdpm"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(parse_err(&value.to_string()))?,
    })
}

/// `raw_data` 是由 [`super::fetch::get_cjpmcx_list`] 返回的原始数据
pub fn rank(json_str: &str) -> Result<Rank, crate::Error<TokenExpired>> {
    let json = crate::hdjw::parse::hdjw_response(json_str)?;
    let raw_data = match json.get("data") {
        Some(data @ Value::Object(_)) => data.clone(),
        _ => return Err(parse_err(json_str)),
    };
    Ok(Rank {
        all: raw_data.get("allPm").map(rank_detail).transpose()?,
        must: raw_data.get("bxkcPm").map(rank_detail).transpose()?,
        core: raw_data.get("hxkcPm").map(rank_detail).transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number(&Value::from(83.42)), Some("83.42".to_string()));
        assert_eq!(parse_number(&Value::from(90)), Some("90".to_string()));
        assert_eq!(parse_number(&Value::from("3.0")), Some("3.0".to_string()));
    }

    #[test]
    fn test_rank() -> TestResult<()> {
        let rank = rank(include_str!("test_data/cjpmcx_list.json"))?;

        let core = rank
            .core
            .expect("`rank.core` should be parsed successfully");
        assert_eq!(core.arithmetic, "85");
        assert_eq!(core.arithmetic_rank, "34/90");
        assert_eq!(core.weighted, "87");
        assert_eq!(core.weighted_rank, "32/90");
        assert_eq!(core.gpa, "3.01");
        assert_eq!(core.gpa_rank, "30/90");

        let must = rank
            .must
            .expect("`rank.must` should be parsed successfully");
        assert_eq!(must.arithmetic, "86");
        assert_eq!(must.arithmetic_rank, "40/90");
        assert_eq!(must.weighted, "88");
        assert_eq!(must.weighted_rank, "38/90");
        assert_eq!(must.gpa, "3.03");
        assert_eq!(must.gpa_rank, "36/90");

        let all = rank.all.expect("`rank.all` should be parsed successfully");
        assert_eq!(all.arithmetic, "87");
        assert_eq!(all.arithmetic_rank, "46/90");
        assert_eq!(all.weighted, "89");
        assert_eq!(all.weighted_rank, "44/90");
        assert_eq!(all.gpa, "3.05");
        assert_eq!(all.gpa_rank, "42/90");

        Ok(())
    }

    #[test]
    fn test_rank_empty() -> TestResult<()> {
        let rank = rank("{}")?;

        assert!(rank.core.is_none());
        assert!(rank.must.is_none());
        assert!(rank.all.is_none());

        Ok(())
    }
}
