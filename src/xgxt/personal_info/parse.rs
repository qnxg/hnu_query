use super::dormitory::parse_dormitory;
use super::{Gender, Level, PersonalInfo};
use crate::error::{MapParseErr, parse_err, parse_err_with_reason};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;

/// 将学工系统接口返回数据中的 `data.groupFields[0].fields` 数组解析为一个 `HashMap`。
/// 具体格式可参考 test_data/ 目录中的样例文件。
pub fn extract_xgxt_entry(
    data: Value,
) -> Result<HashMap<String, String>, crate::Error<Infallible>> {
    let mut parsed_entries = HashMap::<String, String>::new();

    data.get("data")
        .and_then(|data| data.get("groupFields"))
        .and_then(|group_field_list| group_field_list.get(0))
        .and_then(|group_field_item| group_field_item.get("fields"))
        .and_then(|fields| fields.as_array())
        .ok_or(parse_err(&data.to_string()))?
        .iter()
        .for_each(|field| {
            if let Some(field_name) = field.get("fieldName")
                && let Some(value) = field.get("defaultValue")
            {
                let Some(field_name) = field_name.as_str() else {
                    return;
                };
                if let Some(v) = value.as_str() {
                    parsed_entries.insert(field_name.to_string(), v.to_string());
                } else if let Some(v) = value.as_i64() {
                    parsed_entries.insert(field_name.to_string(), v.to_string());
                }
            }
        });

    Ok(parsed_entries)
}

/// 将 [extract_xgxt_entry] 中提取出的 `HashMap` 解析为 [PersonalInfo]。
pub fn person_info(
    mut entries: HashMap<String, String>,
) -> Result<PersonalInfo, crate::Error<Infallible>> {
    let entries_str = serde_json::to_string(&entries).expect("序列化失败");

    let name = entries
        .remove("姓名")
        .ok_or(parse_err_with_reason(&entries_str, "name"))?;
    let enter_year: u16 = entries
        .remove("年级")
        .ok_or(parse_err_with_reason(&entries_str, "enter_year"))?
        .parse()
        .parse_err_with_reason(&entries_str, "enter_year")?;
    let xz = entries
        .remove("学制(年)")
        .and_then(|v| {
            if v.is_empty() {
                None
            } else {
                Some(v.parse::<u8>())
            }
        })
        .transpose()
        .parse_err_with_reason(&entries_str, "xz")?;
    let stu_id = entries
        .remove("学号")
        .ok_or(parse_err_with_reason(&entries_str, "stu_id"))?;
    let gender = match entries.get("性别").map(|v| v.as_str()) {
        Some("1") => Gender::Male,
        Some("2") => Gender::Female,
        _ => {
            return Err(parse_err_with_reason(&entries_str, "gender"))?;
        }
    };
    let level = match entries
        .remove("培养层次")
        .ok_or(parse_err_with_reason(&entries_str, "level"))?
        .as_ref()
    {
        "1" => Level::Doctoral,
        "2" => Level::Postgraduate,
        "3" => Level::Undergraduate,
        _ => {
            return Err(parse_err_with_reason(&entries_str, "level"))?;
        }
    };
    let academy = entries
        .remove("学院")
        .ok_or(parse_err_with_reason(&entries_str, "academy"))?;
    let major = entries
        .remove("专业")
        .ok_or(parse_err_with_reason(&entries_str, "major"))?;
    let class = entries
        .remove("班级")
        .ok_or(parse_err_with_reason(&entries_str, "class"))?;
    let dormitory = entries
        .remove("寝室楼")
        .ok_or(parse_err_with_reason(&entries_str, "dormitory"))?;
    let room = entries
        .remove("寝室号")
        .ok_or(parse_err_with_reason(&entries_str, "room"))?;
    let dormitory = if dormitory.is_empty() || room.is_empty() {
        None
    } else {
        Some(parse_dormitory(dormitory, room))
    };
    let res = PersonalInfo {
        name,
        enter_year,
        xz,
        stu_id,
        gender,
        level,
        academy,
        major,
        class,
        dormitory,
        politic: entries.remove("政治面貌"),
        race: entries.remove("民族"),
        hometown: entries.remove("籍贯"),
        phone: entries.remove("手机号码"),
        wechat: entries.remove("微信号"),
        qq: entries.remove("QQ号码"),
        email: entries.remove("电子邮箱"),
    };
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_person_info() {
        let raw_data_list = vec![
            include_str!("test_data/user_info.json").to_string(),
            include_str!("test_data/in_school_info.json").to_string(),
            include_str!("test_data/contact_info.json").to_string(),
        ]
        .into_iter()
        .map(|s| serde_json::from_str(&s).expect("准备测试数据时发生意外错误"));

        let mut entries = HashMap::<String, String>::new();
        for raw_data in raw_data_list {
            entries.extend(extract_xgxt_entry(raw_data).expect("准备测试数据时发生意外错误"));
        }

        let info = person_info(entries).expect("xgxt personal_info 解析失败");

        assert_eq!(info.name, "林政和");
        assert_eq!(info.enter_year, 2025);
        assert_eq!(info.xz, Some(4));
        assert_eq!(info.stu_id, "202506050175");
        assert_eq!(info.gender, Gender::Male);
        assert_eq!(info.level, Level::Undergraduate);
        assert_eq!(info.academy, "0004");
        assert_eq!(info.major, "0605");
        assert_eq!(info.class, "2025060501");
        assert_eq!(info.politic, Some("".to_string()));
        assert_eq!(info.race, Some("01".to_string()));
        assert_eq!(info.hometown, Some("430104".to_string()));
        assert_eq!(info.phone, Some("13000000000".to_string()));
        assert_eq!(info.wechat, Some("my_wechat".to_string()));
        assert_eq!(info.qq, Some("123456".to_string()));
        assert_eq!(info.email, Some("qnxg@example.com".to_string()));

        let dorm = info.dormitory.expect("xgxt personal_info 宿舍解析失败");
        assert!(dorm.successfully_parsed());
        assert_eq!(dorm.park(), Some("天马园区"));
        assert_eq!(dorm.build(), Some("三区13栋"));
        assert_eq!(dorm.room(), "123");
    }
}
