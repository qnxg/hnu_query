use crate::iportal::{error::IPortalExpired, term::TermInfo, util::iportal_jsondata_precheck};

pub fn parse_term_info(json_str: &str) -> Result<TermInfo, crate::Error<IPortalExpired>> {
    let term_info: TermInfo = serde_json::from_value(iportal_jsondata_precheck(json_str)?)
        .map_err(|e| crate::error::parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
    Ok(term_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_term_info() -> TestResult<()> {
        let term = parse_term_info(include_str!("test_data/term.json"))?;

        assert_eq!(term.year, "2025-2026");
        assert_eq!(term.week, 9);
        assert_eq!(term.term, "4");
        assert_eq!(term.description, "夏季");
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
}
