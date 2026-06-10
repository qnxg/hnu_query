use crate::{
    error::{MapParseErr, parse_err},
    hdjw::{error::TokenExpired, exam_schedule::raw::ExamScheduleItem},
};
use chrono::NaiveDate;

use super::ExamSchedule;

pub fn exam_schedule(
    raw_data: Vec<ExamScheduleItem>,
) -> Result<Vec<ExamSchedule>, crate::Error<TokenExpired>> {
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let (date, time) = match item.kssj {
            Some(kssj) => {
                let [date, time] = kssj.split(' ').collect::<Vec<_>>()[..] else {
                    return Err(parse_err(&kssj));
                };
                let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").parse_err(date)?;
                (Some(date), Some(time.to_string()))
            }
            None => (None, None),
        };

        res.push(ExamSchedule {
            course_id: item.kch,
            course_name: item.kskcmc,
            area: item.ksxq,
            classroom: item.js_mc,
            date,
            time,
            seat: item.zwh,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_exam_schedule() -> TestResult<()> {
        let raw_data: Vec<ExamScheduleItem> =
            serde_json::from_str(include_str!("test_data/xsksap_list.json"))?;

        let schedules = exam_schedule(raw_data)?;

        assert_eq!(schedules.len(), 6);

        let first_item = &schedules[0];
        assert_eq!(first_item.course_id, "TB006MY24");
        assert_eq!(first_item.course_name, "思想道德与法治");
        assert_eq!(
            first_item.date,
            Some(NaiveDate::from_ymd_opt(2026, 6, 23).expect("this should not panic"))
        );
        assert_eq!(first_item.time, Some("15:00~17:00".to_string()));
        assert_eq!(first_item.area, Some("南校区(天马)".to_string()));
        assert_eq!(first_item.classroom, Some("综301".to_string()));
        assert_eq!(first_item.seat, Some("1".to_string()));

        Ok(())
    }
}
