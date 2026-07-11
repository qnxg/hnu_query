use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    gym::{error::TokenExpired, login::GymToken},
    utils::client,
};

const DETAIL_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getSchoolFitClassDetail";
const APPOINT_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getStudentClass";

pub async fn appointment_list(gym_token: &GymToken) -> Result<String, crate::Error<TokenExpired>> {
    client
        .post(APPOINT_URL)
        .headers(gym_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}

/// `class_id`, `class_time`, `test_time` 均为 [super::parse::appointment_list]
/// 返回的 [RawAppointment] 中的字段
pub async fn appointment_detail(
    gym_token: &GymToken,
    class_id: u32,
    class_time: &str,
    test_time: &str,
) -> Result<String, crate::Error<TokenExpired>> {
    client
        .post(DETAIL_URL)
        .form(&[
            ("class_id", class_id.to_string()),
            ("class_time", class_time.to_string()),
            ("test_time", test_time.to_string()),
        ])
        .headers(gym_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}
