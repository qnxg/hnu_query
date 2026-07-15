use super::EmptyClassroom;
use crate::{error::parse_err, hdjw::error::TokenExpired};

/// # Arguments
///
/// - `json_str` 为 [`super::fetch::empty_classroom`] 返回的数据
/// - `time` 为选中的节次。只会将选中的节次的空教室信息解析出来
pub fn empty_classroom(
    json_str: &str,
    time: &[u8],
) -> Result<Vec<EmptyClassroom>, crate::Error<TokenExpired>> {
    let raw_data = crate::hdjw::parse::hdjw_response(json_str)?;
    let data = raw_data
        .as_array()
        .and_then(|v| v.get(4))
        .and_then(|v| v.as_array())
        .ok_or(parse_err("无法解析空教室信息", &raw_data.to_string()))?;
    let mut res = Vec::new();
    for item in data {
        let item = item
            .as_array()
            .ok_or(parse_err("无法解析空教室信息", &item.to_string()))?;
        let mut is_free = true;
        // 需要每一节课均为空才会被认为是空教室
        for i in 1..=time.len() {
            if !item
                .get(i)
                .ok_or(parse_err("无法解析空教室占用情况", &format!("{:?}", item)))?
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
            return Err(parse_err("解析空教室具体信息失败", &format!("{:?}", item)));
        };

        if seat_count_str.len() < 3
            || !seat_count_str.starts_with('(')
            || !seat_count_str.ends_with(')')
        {
            return Err(parse_err("无法解析座位数", seat_count_str));
        }
        let [Ok(seat_count), Ok(exam_seat_count)] = seat_count_str[1..seat_count_str.len() - 1]
            .split('/')
            .map(|x| x.parse::<u32>())
            .collect::<Vec<_>>()[..]
        else {
            return Err(parse_err("无法解析座位数", seat_count_str));
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
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_empty_classroom() -> TestResult<()> {
        fn assert_classrooms(empty_classrooms: &[EmptyClassroom], expected: Vec<&str>) {
            let mut empty_classrooms_names: Vec<&String> =
                empty_classrooms.iter().map(|r| &r.room_name).collect();
            empty_classrooms_names.sort();

            assert_eq!(empty_classrooms_names, expected)
        }

        let rooms = empty_classroom(include_str!("test_data/jsjy_query2.json"), &[1, 2, 3, 4, 5])?;
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
