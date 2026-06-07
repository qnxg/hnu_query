use crate::error::parse_err;
use regex::RegexBuilder;
use std::convert::Infallible;

use super::Rank;

/// # Parameters
///
/// - `pdf_text`: 由 [super::raw::certification_pdf_text] 从 PDF 中提取的文本内容
pub fn rank(pdf_text: &str) -> Result<Rank, crate::Error<Infallible>> {
    let regex = RegexBuilder::new(r"平均学分绩点排名 ([0-9/]+).*平均学分绩点 ([0-9.]+).*核心课程平均学分绩点排名 ([0-9/]+).*必修课平均学分绩点 ([0-9.]+).*课程算术平均成绩排名 ([0-9/]+).*算术平均分 ([0-9.]+).*核心课程算术平均成绩排名 ([0-9/]+).*必修课算术平均分 ([0-9.]+).*学分加权平均成绩排名 ([0-9/]+).*加权平均分 ([0-9.]+).*核心课程学分加权平均成绩排名 ([0-9/]+).*必修课加权平均分 ([0-9.]+)")
        .dot_matches_new_line(true)
        .build()
        .unwrap_or_else(|e| panic!("构建正则表达式失败: {:?}", e));

    let caps = regex
        .captures(pdf_text)
        .ok_or(parse_err(pdf_text))?
        .iter()
        .map(|c| c.map(|v| v.as_str().to_string()).ok_or(parse_err(pdf_text)))
        .collect::<Result<Vec<_>, _>>()?;
    // 12 个捕获组，caps[0] 是完整匹配，共 13 个
    let [
        _,
        all_gpa_rank,
        all_gpa,
        core_gpa_rank,
        must_gpa,
        all_arithmetic_rank,
        all_arithmetic,
        core_arithmetic_rank,
        must_arithmetic,
        all_weighted_rank,
        all_weighted,
        core_weighted_rank,
        must_weighted,
    ] = caps.try_into().map_err(|_| parse_err(pdf_text))?;
    let res = Rank {
        all_gpa,
        all_gpa_rank,
        all_weighted,
        all_weighted_rank,
        all_arithmetic,
        all_arithmetic_rank,
        must_gpa,
        must_weighted,
        must_arithmetic,
        core_gpa_rank,
        core_arithmetic_rank,
        core_weighted_rank,
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_parse_grade_rank() -> TestResult<()> {
        let pdf_text = include_str!("test_data/grade_rank_pdf_extracted.txt");
        let rank = rank(pdf_text)?;

        assert_eq!(rank.all_gpa_rank, "30/90");
        assert_eq!(rank.all_gpa, "4.0");
        assert_eq!(rank.core_gpa_rank, "31/90");
        assert_eq!(rank.must_gpa, "3.8");
        assert_eq!(rank.all_arithmetic_rank, "32/90");
        assert_eq!(rank.all_arithmetic, "90");
        assert_eq!(rank.core_arithmetic_rank, "33/90");
        assert_eq!(rank.must_arithmetic, "88");
        assert_eq!(rank.all_weighted_rank, "34/90");
        assert_eq!(rank.all_weighted, "86");
        assert_eq!(rank.core_weighted_rank, "35/90");
        assert_eq!(rank.must_weighted, "84");

        Ok(())
    }
}
