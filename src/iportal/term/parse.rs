use std::error::Error as StdError;

use crate::{
    error::{MapParseErr, parse_err},
    iportal::{
        term::{
            TermInfo,
            TermType::{self},
        },
        util::iportal_jsondata_precheck,
    },
};
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[expect(unused)]
struct RawTermInfo {
    start_date: String,
    end_date: String,
    dsc: String,
    term: String,
    year: String,
    week: u16,
}

/// 解析学期信息响应
///
/// # Arguments
///
/// - `json_str`: [`super::fetch::term_info`] 返回的数据
pub fn term_info<E: StdError>(json_str: &str) -> Result<TermInfo, crate::Error<E>> {
    let raw: RawTermInfo =
        serde_json::from_value(iportal_jsondata_precheck(json_str)?).parse_err(json_str)?;
    let parse_date = |value: &str| {
        NaiveDateTime::parse_from_str(&format!("{value} 00:00:00"), "%Y-%m-%d %H:%M:%S")
            .parse_err(json_str)
    };
    Ok(TermInfo {
        start_date: parse_date(&raw.start_date)?,
        end_date: parse_date(&raw.end_date)?,
        //description: raw.dsc,
        term: match raw.term.as_str() {
            "1" => TermType::Autumn,
            "2" => TermType::WinterVacation,
            "3" => TermType::Spring,
            "4" => TermType::SummerVacation,
            _ => return Err(parse_err("未知学期类型", json_str)),
        },
        year: raw.year,
        week: raw.week,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParseError, test::TestResult};

    #[test]
    fn test_parse_term_info() -> TestResult<()> {
        let term = term_info::<ParseError>(include_str!("test_data/term.json"))?;

        assert_eq!(term.year, "2025-2026");
        assert_eq!(term.week, 9);
        assert_eq!(term.term, TermType::SummerVacation);
        assert_eq!(
            term.start_date,
            chrono::NaiveDateTime::parse_from_str("2026-07-05 00:00:00", "%Y-%m-%d %H:%M:%S")?
        );
        assert_eq!(
            term.end_date,
            chrono::NaiveDateTime::parse_from_str("2026-09-12 00:00:00", "%Y-%m-%d %H:%M:%S")?
        );

        Ok(())
    }

    #[test]
    fn test_term_info_invalid_date() -> TestResult<()> {
        let result = term_info::<ParseError>(include_str!("test_data/term_invalid_date.json"));

        assert!(result.is_err());
        Ok(())
    }
}
