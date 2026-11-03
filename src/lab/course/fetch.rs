use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    lab::login::LabToken,
    utils::client,
};
use std::{collections::HashMap, convert::Infallible};

const COURSE_LIST_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentScoreList";

pub async fn course_list(
    lab_token: &LabToken,
    semester_id: &str,
) -> Result<String, crate::Error<Infallible>> {
    let mut form_data = HashMap::new();
    form_data.insert("page", "1");
    form_data.insert("rows", "15");
    form_data.insert("SemID", semester_id);
    form_data.insert("UserID", lab_token.stu_id());
    client
        .post(COURSE_LIST_URL)
        .form(&form_data)
        .headers(lab_token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
