use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    gym::{
        error::TokenExpired,
        login::GymToken,
        raw::{GymResponse, GymResponseExtractor},
    },
    utils::client,
};
use serde::Deserialize;

const DETAIL_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getSchoolFitClassDetail";
const APPOINT_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getStudentClass";

#[derive(Deserialize, Debug)]
pub struct AppointmentItem {
    pub class_id: u32,
    pub button_status: i32,
    pub class_name: String,
    /// 如：2025-12-15
    pub class_time: String,
    /// 如：2025年12月15号（周一）
    pub show_time: String,
    /// 如：10:00 - 11:30
    pub test_time: String,
}

#[derive(Deserialize, Debug)]
pub struct AppointmentDetail {
    pub class_desc: String,
    pub appo_type: i32,
}

pub async fn raw_appointment_list_data(
    gym_token: &GymToken,
) -> Result<Vec<AppointmentItem>, crate::Error<TokenExpired>> {
    let gym_headers = gym_token.headers().clone();

    client
        .post(APPOINT_URL)
        .headers(gym_headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data::<Vec<AppointmentItem>, TokenExpired>()
        .await?
        .check_cache()?
        .into_result()
}

/// 获取体测预约详情
///
/// # Arguments
///
/// - `class_id`, `class_time`, `test_time` 均为 `raw_appointment_list_data` 返回的 `AppointmentItem` 中的字段
pub async fn raw_appointment_detail_data(
    gym_token: &GymToken,
    class_id: u32,
    class_time: &str,
    test_time: &str,
) -> Result<AppointmentDetail, crate::Error<TokenExpired>> {
    let gym_headers = gym_token.headers().clone();
    client
        .post(DETAIL_URL)
        .form(&[
            ("class_id", class_id.to_string()),
            ("class_time", class_time.to_string()),
            ("test_time", test_time.to_string()),
        ])
        .headers(gym_headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data::<AppointmentDetail, TokenExpired>()
        .await?
        .check_cache()?
        .into_result()
}
