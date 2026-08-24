use crate::{
    cg::{login::CgSession, login::CgToken, login::LoginError},
    test::{TEST_CACHE, TestResult},
};
use reqwest::header::COOKIE;
use std::io::{self, Read, Write};

/// 获取 CG 系统令牌
///
/// 需要设置环境变量 `TEST_STU_ID` 和 `TEST_CG_PASSWORD`。
/// 运行时会自动下载验证码图片，保存目录由 `TEST_CG_CAPTCHA_DIR` 指定。
///
/// 如果设置了 `TEST_CACHE`，会缓存令牌到 `cache/cg/` 目录，后续无需重复输入验证码。
pub async fn get_cg_token() -> TestResult<CgToken> {
    let stu_id = env!("TEST_STU_ID");
    let password = env!("TEST_CG_PASSWORD");
    let captcha_dir = env!("TEST_CG_CAPTCHA_DIR");

    let cache_name = format!("{:x}", md5::compute(format!("{}{}", stu_id, password)));
    if *TEST_CACHE {
        println!("使用 CgToken 缓存: {}", cache_name);
        std::fs::create_dir_all("cache/cg")?;
        let mut cache_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("cache/cg/{}", cache_name))?;
        let mut cookies = String::new();
        cache_file.read_to_string(&mut cookies)?;
        if !cookies.is_empty() {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(COOKIE, cookies.parse()?);
            return Ok(CgToken::from_headers_unchecked(headers));
        }
    }

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

    let token = match session.login(stu_id, password, captcha_code).await {
        Ok(token) => token,
        Err(e) => match &e {
            crate::Error::Other(LoginError::CaptchaError) => {
                return Err("验证码错误，请重试".into());
            }
            crate::Error::Other(LoginError::PasswordError) => return Err("密码错误".into()),
            _ => return Err(e.to_string().into()),
        },
    };

    if *TEST_CACHE {
        std::fs::create_dir_all("cache/cg")?;
        let cookies = token
            .headers()
            .get(COOKIE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let mut cache_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("cache/cg/{}", cache_name))?;
        cache_file.write_all(cookies.as_bytes())?;
    }

    Ok(token)
}
