mod captcha;

use crate::{
    lab::{
        login::{LabToken, LoginIssue},
        test::captcha::TestCaptchaResolver,
    },
    test::TEST_STU_ID,
};
use std::sync::LazyLock;

pub static TEST_SEMESTER_ID: &str = env!("TEST_LAB_SEMESTER_ID");

pub static TEST_COURSE_ID: &str = env!("TEST_LAB_COURSE_ID");

pub static TEST_LAB_MAX_TRIED: LazyLock<usize> =
    LazyLock::new(|| env!("TEST_LAB_MAX_TRIED").parse().unwrap());

pub static TEST_LAB_PASSWORD: &str = env!("TEST_LAB_PASSWORD");

pub async fn get_lab_token() -> Result<LabToken, crate::Error<LoginIssue>> {
    let captcha_resolver = TestCaptchaResolver;
    LabToken::acquire_by_login(
        TEST_STU_ID,
        TEST_LAB_PASSWORD,
        &captcha_resolver,
        *TEST_LAB_MAX_TRIED,
    )
    .await
}
