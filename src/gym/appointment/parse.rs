use super::Appointment;
use crate::{
    error::MapParseErr,
    gym::{
        appointment::raw::{RawAppointmentDetail, RawAppointmentItem},
        error::TokenExpired,
    },
};
use chrono::NaiveDate;

pub fn appointment_item(
    raw_item: RawAppointmentItem,
    raw_detail: RawAppointmentDetail,
) -> Result<Appointment, crate::Error<TokenExpired>> {
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
