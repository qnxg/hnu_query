use crate::{
    error::{parse_err, parse_err_with_reason},
    hdjw::error::TokenExpired,
};
use serde_json::Value;

use super::EmptyClassroom;

/// # Parameters
///
/// - `raw_data`: 由 [`super::raw::get_jsjy_query2`] 返回的原始数据
pub fn empty_classroom(
    raw_data: Value,
    time: &[u8],
) -> Result<Vec<EmptyClassroom>, crate::Error<TokenExpired>> {
    let data = raw_data
        .as_array()
        .and_then(|v| v.get(4))
        .and_then(|v| v.as_array())
        .ok_or(parse_err(&raw_data.to_string()))?;
    let mut res = Vec::new();
    for item in data {
        let item = item.as_array().ok_or(parse_err(&item.to_string()))?;
        let mut is_free = true;
        // 需要每一节课均为空才会被认为是空教室
        for i in 1..=time.len() {
            if !item
                .get(i)
                .ok_or(parse_err_with_reason(
                    &format!("{:?}", item),
                    "空教室占用情况",
                ))?
                .is_null()
            {
                is_free = false;
                break;
            }
        }
        if !is_free {
            continue;
        }

        let (Some(room_name), Some(seat_count_str), Some(room_type)) = (
            item.first().and_then(|v| v.as_str()),
            item.get(2 + time.len()).and_then(|v| v.as_str()),
            item.get(3 + time.len()).and_then(|v| v.as_str()),
        ) else {
            return Err(parse_err_with_reason(&format!("{:?}", item), "空教室信息"));
        };

        if seat_count_str.len() < 3
            || !seat_count_str.starts_with('(')
            || !seat_count_str.ends_with(')')
        {
            return Err(parse_err_with_reason(seat_count_str, "座位数"));
        }
        let [Ok(seat_count), Ok(exam_seat_count)] = seat_count_str[1..seat_count_str.len() - 1]
            .split('/')
            .map(|x| x.parse::<u32>())
            .collect::<Vec<_>>()[..]
        else {
            return Err(parse_err_with_reason(seat_count_str, "座位数"));
        };
        res.push(EmptyClassroom {
            room_name: room_name.to_string(),
            room_type: room_type.to_string(),
            seat_count,
            exam_seat_count,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_empty_classroom() -> TestResult<()> {
        let raw_data: Value = serde_json::from_str(include_str!("test_data/jsjy_query2.json"))?;

        fn assert_classrooms(empty_classrooms: &[EmptyClassroom], expected: Vec<&str>) {
            let mut empty_classrooms_names: Vec<&String> =
                empty_classrooms.iter().map(|r| &r.room_name).collect();
            empty_classrooms_names.sort();

            assert_eq!(empty_classrooms_names, expected)
        }

        let rooms = empty_classroom(raw_data.clone(), &[1, 2, 3, 4, 5])?;
        assert_classrooms(&rooms, vec!["综B103", "综B104", "综B105", "综B109"]);

        // 教室信息在此处测试，单个通过即认为 OK
        let room = &rooms[0];
        assert_eq!(room.room_name, "综B103");
        assert_eq!(room.seat_count, 70);
        assert_eq!(room.exam_seat_count, 51);
        assert_eq!(room.room_type, "计算机房");

        Ok(())
    }
}
