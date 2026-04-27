use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    gym::{
        error::TokenExpired,
        grade::utils::none_to_zero,
        login::GymToken,
        raw::{GymResponse, GymResponseExtractor},
    },
    utils::client,
};
use serde::Deserialize;

const GRADE_SUMMARY_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Report/getStudentScore";
const GRADE_DETAIL_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Report/getEyeDetails";

/// 体测的摘要成绩
///
/// 仅包含了项目的成绩和等级，没有包含项目具体的数据，也没有总的数据
///
/// see also [`GradeDetail`]
#[derive(Deserialize, Debug)]
pub struct GradeSummary {
    #[serde(rename = "50m_class")]
    pub short_run_class: Option<String>,
    // #[serde(rename = "50m_grade")]
    // pub short_run_grade: String,
    #[serde(rename = "50m_score")]
    #[serde(deserialize_with = "none_to_zero")]
    pub short_run_score: Option<String>,
    pub bmi_class: Option<String>,
    // pub bmi_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub bmi_score: Option<String>,
    pub jump_class: Option<String>,
    // pub jump_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub jump_score: Option<String>,
    // pub lack_show_score_msg: f64,
    pub pull_and_sit_class: Option<String>,
    // pub pull_and_sit_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub pull_and_sit_score: Option<String>,
    pub run_class: Option<String>,
    // pub run_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub run_score: Option<String>,
    pub sit_and_reach_class: Option<String>,
    #[serde(deserialize_with = "none_to_zero")]
    pub sit_and_reach_score: Option<String>,
    // pub student_name: String,
    // pub student_num: String,
    // pub total_grade: String,
    // pub total_score: f64,
    pub vc_class: Option<String>,
    // pub vc_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub vc_score: Option<String>,
    pub report_desc: Option<String>,
    pub report_status: Option<String>,
    pub report_type: Option<String>,
}

/// 体测的详细成绩
///
/// 包含了项目成绩的具体数据和成绩，但是没有包含项目成绩的等级
///
/// 同时还包含了视力成绩，总成绩，姓名学号等数据
#[derive(Deserialize, Debug)]
pub struct GradeDetail {
    pub eyesight_right: String,
    pub eyesight_left: String,
    pub eye_mirror_right: String,
    pub eye_mirror_left: String,
    pub eye_ametropia_right: String,
    pub eye_ametropia_left: String,
    pub bmi_score: i32,
    pub vc_score: i32,
    pub jump_score: i32,
    pub sit_and_reach_score: i32,
    pub pull_and_sit_score: i32,
    #[serde(rename = "50m_score")]
    pub short_run_score: i32,
    pub run_score: i32,
    pub total_score: f64,
    pub total_grade: String,
    pub extra_score_pull_or_sit_up: i32,
    pub extra_score_run: i32,
    pub eyesight_right_detail: String,
    pub eyesight_left_detail: String,
    pub eye_mirror_right_detail: String,
    pub eye_mirror_left_detail: String,
    pub eye_ametropia_right_detail: String,
    pub eye_ametropia_left_detail: String,
    pub student_name: String,
    pub student_num: String,
    pub bmi_grade: String,
    pub jump: String,
    pub jump_grade: String,
    pub pull_and_sit: i32,
    pub pull_and_sit_grade: String,
    #[serde(rename = "50m")]
    pub short_run: String,
    #[serde(rename = "50m_grade")]
    pub short_run_grade: String,
    pub run: String,
    pub run_grade: String,
    pub sit_and_reach: String,
    pub sit_and_reach_grade: String,
    pub vc: i32,
    pub vc_grade: String,
    pub height: String,
    pub weight: String,
    // TODO 添加更新时间
    // pub update_at: String,
}

/// 获取体测的摘要成绩
pub async fn raw_grade_summary_data(
    gym_token: &GymToken,
    xn: u16,
) -> Result<GradeSummary, crate::Error<TokenExpired>> {
    let gym_headers = gym_token.headers().clone();
    client
        .post(GRADE_SUMMARY_URL)
        .form(&[("year_num", xn)])
        .headers(gym_headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data::<GradeSummary, TokenExpired>()
        .await?
        .check_cache()?
        .into_result()
}

pub async fn raw_grade_detail_data(
    gym_token: &GymToken,
    xn: u16,
) -> Result<GradeDetail, crate::Error<TokenExpired>> {
    let gym_headers = gym_token.headers().clone();
    client
        .post(GRADE_DETAIL_URL)
        .form(&[("year_num", xn)])
        .headers(gym_headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data::<GradeDetail, TokenExpired>()
        .await?
        .check_cache()?
        .into_result()
}
