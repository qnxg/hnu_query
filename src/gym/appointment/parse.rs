use super::Appointment;
use crate::{error::MapParseErr, gym::error::TokenExpired};
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RawAppointment {
    pub class_id: u32,
    button_status: i32,
    class_name: String,
    /// 如：2025-12-15
    pub class_time: String,
    /// 如：2025年12月15号（周一）
    show_time: String,
    /// 如：10:00 - 11:30
    pub test_time: String,
}

/// `json_str` 为 [super::fetch::appointment_list] 返回的响应字符串
///
/// 仅将响应字符串解析成 [RawAppointment] 列表，需要再通过 [super::fetch::appointment_detail] 获取预约详情后继续解析
pub fn appointment_list(json_str: &str) -> Result<Vec<RawAppointment>, crate::Error<TokenExpired>> {
    let json = crate::gym::parse::gym_response(json_str)?;
    serde_json::from_value::<Vec<RawAppointment>>(json).parse_err(json_str)
}

#[derive(Deserialize, Debug)]
struct RawAppointmentDetail {
    class_desc: String,
    appo_type: i32,
}

/// # Parameters
///
/// - `raw_item` 为 [super::fetch::appointment_list] 返回的 [RawAppointment]
/// - `detail_str` 为 [super::fetch::appointment_detail] 返回的响应字符串
pub fn appointment_item(
    raw_item: RawAppointment,
    detail_str: &str,
) -> Result<Appointment, crate::Error<TokenExpired>> {
    let detail_json = crate::gym::parse::gym_response(detail_str)?;
    let raw_detail =
        serde_json::from_value::<RawAppointmentDetail>(detail_json).parse_err(detail_str)?;
    Ok(Appointment {
        name: raw_item.class_name,
        desc: raw_detail.class_desc,
        show_date: raw_item.show_time,
        date: NaiveDate::parse_from_str(&raw_item.class_time, "%Y-%m-%d")
            .parse_err(&raw_item.class_time)?,
        time: raw_item.test_time,
        test_type: raw_detail.appo_type,
        status: raw_item.button_status,
    })
}
