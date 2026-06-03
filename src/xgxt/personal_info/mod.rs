mod dormitory;
mod raw;

use crate::{
    error::{MapParseErr, parse_err, parse_err_with_reason},
    xgxt::{
        login::XgxtToken,
        personal_info::{
            dormitory::parse_dormitory,
            raw::{raw_contact_info, raw_in_school_info, raw_user_info},
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use tokio::try_join;

pub use dormitory::Dormitory;

/// 培养层次
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Level {
    /// 本科
    Undergraduate,
    /// 硕士研究生
    Postgraduate,
    /// 博士研究生
    Doctoral,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalInfo {
    /// 姓名
    pub name: String,
    /// 年级（入学年份应该与年级相等），如 `2024`
    pub enter_year: u16,
    /// 学制，如 `4`
    ///
    /// 硕士和博士可能学制比较弹性，因此学工系统中没有学制信息，这个字段是 `None`
    pub xz: Option<u8>,
    /// 学号
    pub stu_id: String,
    /// 性别
    pub gender: Gender,
    /// 培养层次
    pub level: Level,
    /// 学院
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub academy: String,
    /// 专业
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub major: String,
    /// 班级
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub class: String,
    /// 宿舍信息
    ///
    /// 一些人没有宿舍信息，原因不明
    pub dormitory: Option<Dormitory>,
    /// 政治面貌
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub politic: Option<String>,
    /// 民族
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub race: Option<String>,
    /// 籍贯
    ///
    /// TODO 目前这个字段只有以逗号分割的数字字符串，后续需要进一步解析
    pub hometown: Option<String>,
    /// 手机号
    pub phone: Option<String>,
    /// 微信号
    pub wechat: Option<String>,
    /// qq号
    pub qq: Option<String>,
    /// 电子邮箱
    pub email: Option<String>,
}

/// 性别
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Gender {
    /// 男
    Male,
    /// 女
    Female,
}

/// 将学工系统三个接口返回数据中的 `data.groupFields[0].fields` 数组解析合并为一个 `HashMap`。
/// 具体格式可参考 test_data/ 目录中的样例文件。
///
/// # Errors
///
/// 如果原始数据缺少必要的字段或字段格式不正确，返回 [ParseError](crate::error::Error::ParseError)。
fn extract_xgxt_entry(data: Value) -> Result<HashMap<String, String>, crate::Error<Infallible>> {
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
///
/// # Errors
///
/// 如果原始数据缺少必要的字段或字段格式不正确，返回 [ParseError](crate::error::Error::ParseError)。
fn parse_person_info(
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

/// 从学工系统获取个人信息
///
/// # Parameters
///
/// - `xgxt_token`: 学工系统令牌，可以通过 [XgxtToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 个人信息
///
/// # Performance
///
/// 这个函数大概会同时发起三个请求，且一次请求数据量比较大（学工系统有个接口直接把近十年所有的班级数据全部返回了），所以建议不要频繁调用本函数。个人信息一般没有什么变动，建议做好缓存。
pub async fn get_person_info(
    xgxt_token: &XgxtToken,
) -> Result<PersonalInfo, crate::Error<Infallible>> {
    let raw_data_list = try_join!(
        raw_user_info(xgxt_token),
        raw_in_school_info(xgxt_token),
        raw_contact_info(xgxt_token),
    )
    .map(|(a, b, c)| vec![a, b, c])?;

    let mut entries = HashMap::<String, String>::new();
    for raw_data in raw_data_list {
        entries.extend(extract_xgxt_entry(raw_data)?);
    }

    parse_person_info(entries)
}

#[cfg(test)]
mod test {
    use crate::{test::test_ok, xgxt::test::get_xgxt_token};

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

        let info = parse_person_info(entries).expect("xgxt personal_info 解析失败");

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

    #[tokio::test]
    #[ignore]
    async fn test_get_person_info() {
        let xgxt_token = get_xgxt_token().await;
        let personal_info = test_ok(get_person_info(&xgxt_token).await, "get personal info");
        println!("{:#?}", personal_info);
    }
}
