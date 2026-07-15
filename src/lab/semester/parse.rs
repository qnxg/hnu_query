use crate::{
    error::{MapParseErr, parse_err},
    lab::semester::Semester,
};
use serde::Deserialize;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
struct RawSemester {
    id: String,
    text: String,
}

/// `json_str` 为 [`super::fetch::semester`] 返回的数据
pub fn semester(json_str: &str) -> Result<Vec<Semester>, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Vec<RawSemester>>(json_str).parse_err(json_str)?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let [xn_str, _, xq_str] = item
            .text
            .split(|c| ['-', '_', ' '].contains(&c))
            .collect::<Vec<&str>>()[..]
        else {
            return Err(parse_err("无法解析学期", &item.text));
        };
        let (Ok(xn), Ok(xq)) = (xn_str.parse::<u16>(), xq_str.parse::<u8>()) else {
            return Err(parse_err("无法解析学期", &item.text));
        };
        res.push(Semester {
            xn,
            xq,
            id: item.id,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_semester() -> TestResult<()> {
        let semesters = semester(include_str!("test_data/GetSemDropDownList.json"))?;
        assert_eq!(semesters.len(), 12);
        // 测试用 _ 分隔的情况
        let first = &semesters[0];
        assert_eq!(first.xn, 2020);
        assert_eq!(first.xq, 3);
        assert_eq!(first.id, "5");
        // 测试用 - 分隔的情况
        let last = &semesters[11];
        assert_eq!(last.xn, 2025);
        assert_eq!(last.xq, 2);
        assert_eq!(last.id, "18");
        Ok(())
    }
}
