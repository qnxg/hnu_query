use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err, parse_err_with_reason},
    utils::client,
    yjsxt::{error::TokenExpired, login::YjsxtToken, utils::YjsxtResponseExtractor},
};

const GRADUATE_HOST_URL: &str = "http://yjsxt.hnu.edu.cn/gmis/";
const CLASS_TABLE_URL: &str = "/student/pygl/py_kbcx_ew";
const BIND_TERM_URL: &str = "/student/default/bindterm";

/// API 返回的学期信息
#[derive(Deserialize, Debug)]
struct TermInfo {
    termcode: String,
    termname: String,
}

/// 研究生课程原始信息
#[derive(Debug)]
pub struct GraduateCourseInfo {
    pub course_id: String,
    pub course_name: String,
    pub teacher: String,
    pub class_name: String,
    pub place: String,
    pub area: String,
    pub day: u8,
    pub sections: String,
    pub weeks: String,
    #[expect(dead_code)]
    pub start_time: String,
    pub end_time: String,
    pub course_type: String,
    pub credit: f32,
    pub extra: Option<String>,
}

struct CourseInfo {
    course_id: String,
    course_name: String,
    class_name: String,
    class_time: String,
    teacher: String,
    classroom: String,
}

fn parse_course_info(input: &str) -> Option<CourseInfo> {
    let parts: Vec<&str> = input.split("<br/>").filter(|s| !s.is_empty()).collect();

    let course_id = parts[0].replace("课程编号:", "").trim().to_string();
    let course_name = parts[1].replace("课程名称:", "").trim().to_string();
    let class_name = parts[2].replace("班级:", "").trim().to_string();
    let class_time = parts
        .iter()
        .find(|s| s.contains("上课时间:"))
        .map(|s| s.replace("上课时间:", "").trim().to_string())?;
    // 原格式: [9-16周] 连续周 → 生成 "9,10,11,...,16"
    let class_time = class_time
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-')
        .collect::<String>();
    let class_time = class_time.split('-').collect::<Vec<&str>>();
    let class_time = (class_time[0].parse::<u8>().unwrap()
        ..=class_time[1].parse::<u8>().unwrap())
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let teacher_and_classroom = parts
        .iter()
        .find(|s| s.contains('[') && !s.starts_with("上课时间:"))
        .map(|s| s.trim())
        .unwrap_or("[]");

    let teacher_end = teacher_and_classroom.chars().position(|c| c == '[');
    let (teacher, classroom) = match teacher_end {
        Some(end) => {
            let t: String = teacher_and_classroom.chars().take(end).collect();
            let classroom_start = end + 1;
            let classroom_end = teacher_and_classroom
                .chars()
                .position(|c| c == ']')
                .unwrap_or(teacher_and_classroom.len());
            let c: String = teacher_and_classroom
                .chars()
                .skip(classroom_start)
                .take(classroom_end - classroom_start)
                .collect();
            (t, c)
        }
        None => (String::new(), String::new()),
    };

    Some(CourseInfo {
        course_id,
        course_name,
        class_name,
        class_time,
        teacher,
        classroom,
    })
}

/// 根据学年学期获取 termcode
async fn get_termcode(
    yjsxt_token: &YjsxtToken,
    xn: u16,
    xq: u8,
) -> Result<u16, crate::Error<TokenExpired>> {
    let url = format!(
        "{}{}{}",
        GRADUATE_HOST_URL,
        yjsxt_token.id(),
        BIND_TERM_URL
    );
    let res = client
        .get(&url)
        .headers(yjsxt_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data(true)
        .await?;

    let terms: Vec<TermInfo> =
        serde_json::from_value(res).parse_err_with_reason("", "解析学期数据失败")?;

    let season_name = match xq {
        1 => "秋学期",
        2 => "春学期",
        3 => "暑假学期",
        _ => return Err(parse_err_with_reason("", &format!("无效学期: {xq}"))),
    };

    let target_termname = format!("{}-{}{}", xn, xn + 1, season_name);

    terms
        .iter()
        .find(|t| t.termname == target_termname)
        .map(|t| t.termcode.parse::<u16>().unwrap())
        .ok_or(parse_err_with_reason("", &format!("未找到对应学期: {target_termname}")))
}

pub async fn raw_class_table_data(
    yjsxt_token: &YjsxtToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<GraduateCourseInfo>, crate::Error<TokenExpired>> {
    let term_code = get_termcode(yjsxt_token, xn, xq).await?;
    let url = format!(
        "{}{}{}",
        GRADUATE_HOST_URL,
        yjsxt_token.id(),
        CLASS_TABLE_URL
    );
    let res: Value = client
        .post(&url)
        .headers(yjsxt_token.headers().clone())
        .form(&[
            ("kblx", "xs"),
            ("termcode", &term_code.to_string()),
        ])
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data(true)
        .await?;

    let rows = res["rows"]
        .as_array()
        .ok_or(parse_err(&serde_json::to_string(&res).unwrap_or_default()))?;

    let start_times: HashMap<u8, &str> = [
        (1, "8:00"),
        (2, "8:55"),
        (3, "10:00"),
        (4, "10:55"),
        (5, "14:30"),
        (6, "15:15"),
        (7, "16:10"),
        (8, "16:55"),
        (9, "19:00"),
        (10, "19:55"),
        (11, "20:50"),
        (12, "21:35"),
    ]
    .into_iter()
    .collect();

    let end_times: HashMap<u8, &str> = [
        (1, "8:45"),
        (2, "9:40"),
        (3, "10:45"),
        (4, "11:40"),
        (5, "15:15"),
        (6, "16:00"),
        (7, "16:55"),
        (8, "17:40"),
        (9, "19:45"),
        (10, "20:40"),
        (11, "21:35"),
        (12, "22:20"),
    ]
    .into_iter()
    .collect();

    let mut courses: Vec<GraduateCourseInfo> = Vec::new();

    for item in rows {
        if item["mc"] == Value::String("无节次".to_string()) {
            continue;
        }
        let jc = item["mc"]
            .as_str()
            .ok_or(parse_err(&serde_json::to_string(&item).unwrap_or_default()))?
            .parse::<u8>()
            .parse_err_with_reason("", "解析节次失败")?;
        let section_id = format!("{:0>2}", jc);

        for day in 1..=7u8 {
            let key = format!("z{day}");
            if item[&key] == Value::Null {
                continue;
            }
            let cell_text = item[&key]
                .as_str()
                .ok_or(parse_err(&serde_json::to_string(&item).unwrap_or_default()))?;
            let course_info =
                parse_course_info(cell_text).ok_or(parse_err(cell_text))?;

            // 尝试与已有课程合并（连续节次）
            let mut merged = false;
            for existing in courses.iter_mut() {
                if existing.course_name == course_info.course_name
                    && existing.weeks == course_info.class_time
                    && existing.teacher == course_info.teacher
                    && existing.day == day
                {
                    let existing_sections: Vec<u8> = existing
                        .sections
                        .split(',')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if existing_sections.contains(&(jc - 1)) {
                        existing.sections =
                            format!("{},{}", existing.sections, section_id);
                        existing.end_time = end_times[&jc].to_string();
                        merged = true;
                        break;
                    }
                }
            }

            if !merged {
                let mut course_id = course_info.course_id.clone();
                let count = courses
                    .iter()
                    .filter(|c| c.course_id.starts_with(&course_info.course_id))
                    .count();
                if count > 0 {
                    course_id = format!("{}_{}", course_id, count + 1);
                }
                courses.push(GraduateCourseInfo {
                    course_id,
                    course_name: course_info.course_name,
                    teacher: course_info.teacher,
                    class_name: course_info.class_name,
                    place: course_info.classroom,
                    area: String::new(),
                    day,
                    sections: section_id.clone(),
                    weeks: course_info.class_time,
                    start_time: start_times[&jc].to_string(),
                    end_time: end_times[&jc].to_string(),
                    course_type: String::new(),
                    credit: 0.0,
                    extra: None,
                });
            }
        }
    }

    Ok(courses)
}
