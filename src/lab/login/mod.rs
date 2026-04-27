mod captcha;
mod utils;

use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::{client, request::cookie_parser},
};
use reqwest::header::{COOKIE, HeaderMap, SET_COOKIE};
use serde_json::Value;

pub use captcha::CaptchaResolver;

const LOGIN_URL: &str = "http://10.62.106.112/BaseInfo/Login/ValidateLogin";
const CAPTCHA_URL: &str = "http://10.62.106.112/Ashx/CheckCode.ashx?t=0.29911677684547566";

/// 大物实验平台的令牌
#[derive(Debug, Clone)]
pub struct LabToken {
    headers: HeaderMap,
    stu_id: String,
}

/// 登录实验平台时可能遇到的错误
#[derive(thiserror::Error, Debug, Clone)]
pub enum LoginIssue {
    /// 密码错误
    #[error("密码错误")]
    PasswordError,
    /// 验证码错误
    ///
    /// 一般是由于你在 [LabToken::acquire_by_login]
    /// 中提供的 `captcha_resolver` 识别率不高，同时 `max_tried`
    /// 参数设置的太小，导致在 `max_tried` 次尝试登录中，每次验证码
    /// 都识别失败了
    #[error("验证码错误")]
    CaptchaError,
    /// 其他错误
    ///
    /// 由大物实验平台返回的无法登录的原因
    #[error("其他错误")]
    OtherError(Option<String>),
}

impl LabToken {
    /// 通过登录获取 [LabToken]
    ///
    /// # Arguments
    ///
    /// - `stu_id`: 学号
    /// - `password`: 该学号对应的大物实验平台的密码
    /// - `captcha_resolver`: 验证码解析器，需要实现 [CaptchaResolver] trait
    /// - `max_tried`: 考虑到验证码解析器识别验证码不太能做到完全正确，
    ///   所以本函数在内部会尝试 `max_tried` 次登录。注意，本函数仅在验证码失败时
    ///   重试，若在其他情况下失败，则直接返回错误
    ///
    /// # Returns
    ///
    /// 返回一个 [LabToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题或是验证码识别失败导致登录失败，此时会返回 [LoginIssue] 错误
    pub async fn acquire_by_login(
        stu_id: &str,
        password: &str,
        captcha_resolver: &impl CaptchaResolver,
        max_tried: usize,
    ) -> Result<Self, crate::Error<LoginIssue>> {
        let password = utils::lab_encrypt(password);
        let mut tried = 0;
        let mut checkcode = String::new();
        let mut all_cookies = String::new();
        let mut loop_result = None;
        while tried < max_tried {
            let res = client
                .post(LOGIN_URL)
                .form(&[
                    ("uname", stu_id),
                    ("pwd", &password),
                    ("checkcode", &checkcode),
                ])
                .header("Cookie", &all_cookies)
                .send()
                .await
                .network_err()?
                .error_for_status()
                .unexpected_err()?;
            let cookies = cookie_parser(res.headers().get_all(SET_COOKIE));
            if !cookies.is_empty() {
                all_cookies.push_str(&format!("; {}", cookies.join("; ")));
            }
            let data_str = res.text().await.unexpected_err()?;
            let data: Value = serde_json::from_str(&data_str).parse_err(&data_str)?;
            let Some(code) = data["RTNCode"].as_i64() else {
                return Err(parse_err(&data_str));
            };
            if code == -2 {
                // 需要验证码
                let res = client
                    .get(CAPTCHA_URL)
                    .header("Cookie", &all_cookies)
                    .send()
                    .await
                    .network_err()?
                    .error_for_status()
                    .unexpected_err()?;
                let img_bytes = res.bytes().await.unexpected_err()?;
                checkcode = captcha_resolver
                    .resolve(img_bytes.as_ref())
                    .await
                    .unexpected_err()?;
                tried += 1;
            } else {
                loop_result = Some((code, data, all_cookies));
                break;
            }
        }
        let Some((code, data, cookies)) = loop_result else {
            return Err(crate::Error::Other(LoginIssue::CaptchaError));
        };
        match code {
            1 => {
                if cookies.is_empty() {
                    Err("Cookie 为空").unexpected_err()
                } else {
                    let mut headers = HeaderMap::new();
                    headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
                    Ok(Self {
                        headers,
                        stu_id: stu_id.to_string(),
                    })
                }
            }
            -1 => Err(crate::Error::Other(LoginIssue::PasswordError)),
            _ => {
                let msg = data.get("Data").and_then(|v| v.as_str().map(String::from));
                Err(crate::Error::Other(LoginIssue::OtherError(msg)))
            }
        }
    }
    /// 从 [HeaderMap] 创建 [LabToken]
    ///
    /// # Arguments
    ///
    /// - `headers`: 一个合法的可用作 [LabToken] 的 [HeaderMap]
    /// - `stu_id`: 该 `headers` 参数对应的学号
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [LabToken] 的 [HeaderMap]，否则会导致未定义行为
    ///
    /// 同时 `stu_id` 参数应该和 `headers` 参数对应，否则也会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap, stu_id: &str) -> Self {
        Self {
            headers,
            stu_id: stu_id.to_string(),
        }
    }
    /// 获取当前令牌的 [HeaderMap]，可用于 [LabToken::from_headers_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌的 [HeaderMap]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    /// 获取当前令牌的学号，可用于 [LabToken::from_headers_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌的学号
    pub fn stu_id(&self) -> &str {
        &self.stu_id
    }
}

#[cfg(test)]
mod test {

    use crate::lab::test::get_lab_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_lab_token() {
        let lab_token = get_lab_token().await.unwrap();
        println!("{:#?}", lab_token);
    }
}
