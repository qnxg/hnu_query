mod dormitory;
mod raw;

use crate::{
    error::{MapParseErr, parse_err_with_reason},
    xgxt::{
        login::XgxtToken,
        personal_info::{dormitory::parse_dormitory, raw::raw_person_info_data},
    },
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

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
    pub dormitory: Dormitory,
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
    let mut entries = raw_person_info_data(xgxt_token).await?;
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
        "1" => Level::Undergraduate,
        "2" => Level::Postgraduate,
        "3" => Level::Doctoral,
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
    let dormitory = parse_dormitory(
        entries
            .remove("寝室楼")
            .ok_or(parse_err_with_reason(&entries_str, "dormitory"))?,
        entries
            .remove("寝室号")
            .ok_or(parse_err_with_reason(&entries_str, "room"))?,
    );

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
    use crate::xgxt::test::get_xgxt_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_person_info() {
        let xgxt_token = get_xgxt_token().await.unwrap();
        let personal_info = get_person_info(&xgxt_token).await.unwrap();
        println!("{:#?}", personal_info);
    }
}
