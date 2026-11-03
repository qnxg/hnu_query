use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    gym::{error::TokenExpired, login::GymToken},
    utils::client,
};

const GRADE_SUMMARY_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Report/getStudentScore";
const GRADE_DETAIL_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Report/getEyeDetails";

pub async fn grade_summary(
    gym_token: &GymToken,
    xn: u16,
) -> Result<String, crate::Error<TokenExpired>> {
    let gym_headers = gym_token.headers().clone();
    client
        .post(GRADE_SUMMARY_URL)
        .form(&[("year_num", xn)])
        .headers(gym_headers)
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}

pub async fn grade_detail(
    gym_token: &GymToken,
    xn: u16,
) -> Result<String, crate::Error<TokenExpired>> {
    let gym_headers = gym_token.headers().clone();
    client
        .post(GRADE_DETAIL_URL)
        .form(&[("year_num", xn)])
        .headers(gym_headers)
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
