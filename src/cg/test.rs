use crate::{
    cg::{CgToken, error::LoginError, login::CgSession},
    test::TestResult,
};
use std::io::{self, Write};

/// 获取 CG 系统令牌
///
/// 需要设置环境变量 `TEST_STU_ID` 和 `TEST_CG_PASSWORD`。
/// 运行时会自动下载验证码图片，保存目录由 `TEST_CG_CAPTCHA_DIR` 指定。
pub async fn get_cg_token() -> TestResult<CgToken> {
    let stu_id = env!("TEST_STU_ID");
    let password = env!("TEST_CG_PASSWORD");
    let captcha_dir = env!("TEST_CG_CAPTCHA_DIR");

    let session = CgSession::new().await?;
    std::fs::create_dir_all(captcha_dir)?;
    let path = format!("{captcha_dir}/cg_captcha.png");
    std::fs::write(&path, session.captcha_image())?;
    println!("验证码已保存到 {path}");

    print!("请输入验证码: ");
    io::stdout().flush()?;
    let mut captcha_code = String::new();
    io::stdin().read_line(&mut captcha_code)?;
    let captcha_code = captcha_code.trim();

    match session.login(stu_id, password, captcha_code).await {
        Ok(token) => Ok(token),
        Err(e) => match &e {
            crate::Error::Other(LoginError::CaptchaError) => Err("验证码错误，请重试".into()),
            crate::Error::Other(LoginError::PasswordError) => Err("密码错误".into()),
            _ => Err(e.to_string().into()),
        },
    }
}
