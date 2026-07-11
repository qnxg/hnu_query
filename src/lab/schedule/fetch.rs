use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    lab::login::LabToken,
    utils::client,
};
use std::{collections::HashMap, convert::Infallible};

const LAB_LIST_URL: &str = "http://10.62.106.112/XPK/StuCourseElectiveLook/LoadTableInfo";

pub async fn lab_schedule(lab_token: &LabToken) -> Result<String, crate::Error<Infallible>> {
    let mut form_data = HashMap::new();
    form_data.insert("CourseID", "-999");
    form_data.insert("weeks", "-999");
    form_data.insert("labID", "-999");
    form_data.insert("page", "1");
    form_data.insert("rows", "200");
    client
        .post(LAB_LIST_URL)
        .form(&form_data)
        .headers(lab_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}
