mod raw;

use crate::{
    hdjw::class_table::{Course, CourseSchedule},
    yjsxt::{error::TokenExpired, login::YjsxtToken},
};
use raw::GraduateCourseInfo;

fn parse_graduate_course_info(
    raw_data: Vec<GraduateCourseInfo>,
) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let mut courses = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let time_list: Vec<u8> = item
            .sections
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        let week_list: Vec<u8> = item
            .weeks
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        let schedule: Vec<CourseSchedule> = week_list
            .into_iter()
            .map(|week| CourseSchedule {
                week,
                day: item.day,
                place: item.place.clone(),
                time: time_list.clone(),
            })
            .collect();
        let course = Course {
            course_name: item.course_name,
            course_id: item.course_id,
            course_type: item.course_type,
            class_name: item.class_name,
            area: item.area,
            teacher: if item.teacher.is_empty() {
                None
            } else {
                Some(item.teacher)
            },
            credit: item.credit,
            extra: item.extra,
            people: 0,
            schedule,
        };
        courses.push(course);
    }
    Ok(courses)
}

/// 获取课表信息
///
/// # Arguments
///
/// * `yjsxt_token` - 研究生系统的令牌，可以通过 [YjsxtToken::acquire_by_cas_login] 获取
/// * `xn` - 学年，如 `2025`
/// * `xq` - 学期，如 `1`
///
/// # Returns
///
/// 返回所选课程的列表
///
/// # Errors
///
/// 如果提供的 `yjsxt_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [YjsxtToken]
pub async fn get_class_table(
    yjsxt_token: &YjsxtToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<Course>, crate::Error<TokenExpired>> {
    let raw_data = raw::raw_class_table_data(yjsxt_token, xn, xq).await?;
    parse_graduate_course_info(raw_data)
}

#[cfg(test)]
mod tests {
    use crate::{
        test::{TEST_XN, TEST_XQ},
        yjsxt::test::get_yjsxt_token,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_get_class_table() {
        let yjsxt_token = get_yjsxt_token().await.unwrap();
        let class_table =
            get_class_table(&yjsxt_token, *TEST_XN, *TEST_XQ).await.unwrap();
        println!("{:#?}", class_table);
    }
}
