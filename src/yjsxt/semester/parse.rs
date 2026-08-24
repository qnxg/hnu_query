use crate::{
    error::{MapParseErr, parse_err},
    yjsxt::{error::TokenExpired, semester::Semester},
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct RawSemester {
    termcode: String,
    termname: String,
}

/// `json_str` 为 [super::fetch::semester] 的返回数据
pub fn semester(json_str: &str) -> Result<Vec<Semester>, crate::Error<TokenExpired>> {
    let raw_data = serde_json::from_str::<Vec<RawSemester>>(json_str).parse_err(json_str)?;
    let mut res = Vec::new();
    for raw_item in raw_data {
        let (xn_str, other) = raw_item
            .termname
            .split_once('-')
            .ok_or_else(|| parse_err("解析学期失败", json_str))?;
        let xn = xn_str.parse::<u16>().parse_err(json_str)?;
        let xq = if other.contains("秋学期") {
            1
        } else if other.contains("春学期") {
            2
        } else if other.contains("暑假学期") {
            3
        } else if other.contains("寒假小学期") {
            4
        } else {
            return Err(parse_err("未知学期", json_str));
        };
        res.push(Semester {
            xn,
            xq,
            id: raw_item.termcode,
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
        // bindterm 接口返回的是明文 JSON，未经过加密
        let semesters = semester(include_str!("test_data/bindterm.json"))?;
        assert_eq!(semesters.len(), 56);
        // 秋学期
        let first = &semesters[0];
        assert_eq!(first.xn, 2026);
        assert_eq!(first.xq, 1);
        assert_eq!(first.id, "63");
        // 暑假学期
        let summer = &semesters[1];
        assert_eq!(summer.xn, 2025);
        assert_eq!(summer.xq, 3);
        assert_eq!(summer.id, "64");
        // 寒假小学期
        let winter = &semesters[2];
        assert_eq!(winter.xn, 2025);
        assert_eq!(winter.xq, 4);
        assert_eq!(winter.id, "62");
        // 春学期
        let spring = &semesters[3];
        assert_eq!(spring.xn, 2025);
        assert_eq!(spring.xq, 2);
        assert_eq!(spring.id, "61");
        // 最早的学期
        let last = &semesters[55];
        assert_eq!(last.xn, 2004);
        assert_eq!(last.xq, 1);
        assert_eq!(last.id, "5");
        Ok(())
    }
}
