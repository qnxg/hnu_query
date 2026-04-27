//! 这里提供了一个用于测试的验证码解析器实现
//!
//! 该解析器依赖于 <https://github.com/qnxg/captcha_service>，
//! 你可以按照仓库的 README 进行部署
use crate::{lab::login::CaptchaResolver, utils::client};
use serde::Deserialize;

const CAPTCHA_SERVICE_URL: &str = "http://localhost:5000";

#[derive(Deserialize, Debug)]
struct CaptchaResponse {
    error: Option<String>,
    result: Option<String>,
}

pub struct TestCaptchaResolver;

impl CaptchaResolver for TestCaptchaResolver {
    async fn resolve(
        &self,
        data: &[u8],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/ocr?type={}", CAPTCHA_SERVICE_URL, "default");
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(data.to_vec()).file_name("captcha.jpg"),
        );
        let res = client.post(&url).multipart(form).send().await?;
        let res = res.error_for_status()?;
        let body = res.text().await?;
        let res: CaptchaResponse = serde_json::from_str(&body)?;
        if let Some(result) = res.result {
            Ok(result)
        } else {
            Err(format!(
                "验证码服务错误: {}",
                res.error.unwrap_or("未知错误".to_string())
            )
            .into())
        }
    }
}
