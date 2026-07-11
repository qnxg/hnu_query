use crate::{
    error::{MapParseErr, parse_err, parse_err_with_reason},
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
    let json_str = crate::yjsxt::parse::decrypt_response(json_str)?;
    let raw_data = serde_json::from_str::<Vec<RawSemester>>(&json_str).parse_err(&json_str)?;
    let mut res = Vec::new();
    for raw_item in raw_data {
        let (xn_str, other) = raw_item
            .termname
            .split_once('-')
            .ok_or_else(|| parse_err(&json_str))?;
        let xn = xn_str.parse::<u16>().parse_err(&json_str)?;
        let xq = if other.contains("秋学期") {
            1
        } else if other.contains("春学期") {
            2
        } else if other.contains("暑假学期") {
            3
        } else {
            return Err(parse_err_with_reason(&json_str, "未知学期"));
        };
        res.push(Semester {
            xn,
            xq,
            id: raw_item.termcode,
        });
    }
    Ok(res)
}
