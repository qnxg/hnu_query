use super::ExamSchedule;
use crate::{
    error::{MapParseErr, parse_err},
    hdjw::error::TokenExpired,
};
use chrono::NaiveDate;
use serde::Deserialize;

// 带 Option 的字段应该是类似于体育理论这样考试安排信息很不全的课程
#[derive(Deserialize, Debug)]
struct RawExamSchedule {
    /// 课程代码
    kch: String,
    /// 课程名称
    kskcmc: String,
    /// 考试校区
    ksxq: Option<String>,
    /// 考试的教室
    js_mc: Option<String>,
    /// 考试时间（已经是一个时间区间了）
    kssj: Option<String>,
    /// 座位号
    zwh: Option<String>,
}

/// `json_str` 为 [`super::fetch::exam_schedule`] 返回的数据
pub fn exam_schedule(json_str: &str) -> Result<Vec<ExamSchedule>, crate::Error<TokenExpired>> {
    let json = crate::hdjw::parse::hdjw_response(json_str)?;
    let raw_data =
        serde_json::from_value::<Vec<RawExamSchedule>>(json["data"].clone()).parse_err(json_str)?;
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
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_exam_schedule() -> TestResult<()> {
        let schedules = exam_schedule(include_str!("test_data/xsksap_list.json"))?;

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
