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

pub fn semester(json_str: &str) -> Result<Vec<Semester>, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Vec<RawSemester>>(json_str).parse_err(json_str)?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let [xn_str, _, xq_str] = item
            .text
            .split(|c| ['-', '_', ' '].contains(&c))
            .collect::<Vec<&str>>()[..]
        else {
            return Err(parse_err(&item.text));
        };
        let (Ok(xn), Ok(xq)) = (xn_str.parse::<u16>(), xq_str.parse::<u8>()) else {
            return Err(parse_err(&item.text));
        };
        res.push(Semester {
            xn,
            xq,
            id: item.id,
        });
    }
    Ok(res)
}
