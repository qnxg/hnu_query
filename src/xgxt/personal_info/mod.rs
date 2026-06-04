mod dormitory;
mod parse;
mod raw;

use crate::xgxt::{
    login::XgxtToken,
    personal_info::raw::{raw_contact_info, raw_in_school_info, raw_user_info},
};
use serde::{Deserialize, Serialize};
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
        entries.extend(parse::extract_xgxt_entry(raw_data)?);
    }

    parse::person_info(entries)
}

#[cfg(test)]
mod test {
    use crate::{test::test_ok, xgxt::test::get_xgxt_token};

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_get_person_info() {
        let xgxt_token = get_xgxt_token().await;
        let personal_info = test_ok(get_person_info(&xgxt_token).await, "get personal info");
        println!("{:#?}", personal_info);
    }
}
