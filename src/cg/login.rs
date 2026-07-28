use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, obs, request::cookie_parser},
};

/// CG 系统登录相关的错误
#[derive(thiserror::Error, Debug, Clone)]
pub enum LoginError {
    /// 验证码错误
    #[error("验证码错误")]
    CaptchaError,
    /// 密码错误
    #[error("密码错误")]
    PasswordError,
    /// 未知登录失败
    #[error("登录失败，错误码: {0}")]
    LoginFailed(String),
}
use aes::cipher::{BlockEncryptMut, KeyInit, block_padding::Pkcs7};
use base64::engine::{Engine, general_purpose::STANDARD as base64};
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};
use std::convert::Infallible;

const BASE_URL: &str = "https://cg.hnu.edu.cn";
const LOGIN_PAGE: &str = "/indexcs/simple.jsp";
const CAPTCHA_URL: &str = "/cgjiaoyan";
const LOGIN_URL: &str = "/login/loginproc.jsp";
const AES_KEY: &str = "Client8Sess!06ID";

type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;

/// CG 前端的加密函数
fn encrypt_password(password: &str) -> Result<String, crate::Error<LoginError>> {
    let key = <aes::cipher::generic_array::GenericArray<u8, _>>::from_slice(AES_KEY.as_bytes());
    let cipher = Aes128EcbEnc::new(key);
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(password.as_bytes());
    Ok(base64.encode(&ciphertext))
}

/// CG 系统的登录会话
#[derive(Debug, Clone)]
pub struct CgSession {
    headers: HeaderMap,
    captcha_image: Vec<u8>,
}

impl CgSession {
    /// 创建一个新的登录会话，同时获取验证码图片
    ///
    /// # Returns
    ///
    /// 返回 [CgSession]，其中包含验证码图片的字节数据。
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn new() -> Result<Self, crate::Error<Infallible>> {
        // 1. 访问登录页面，获取 JSESSIONID
        let res = client
            .get(format!("{}{}", BASE_URL, LOGIN_PAGE))
            .send()
            .await
            .network_err()?;
        let cookies = cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ");
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);

        // 2. 下载验证码图片
        let captcha_image = client
            .get(format!("{}{}", BASE_URL, CAPTCHA_URL))
            .headers(headers.clone())
            .send()
            .await
            .network_err()?
            .bytes()
            .await
            .unexpected_err()?
            .to_vec();

        Ok(Self {
            headers,
            captcha_image,
        })
    }

    /// 获取验证码图片的字节数据
    ///
    /// 可以写入文件后查看，例如：
    ///
    /// ```no_run
    /// # use hnu_query::cg::login::CgSession;
    /// # async {
    /// let session = CgSession::new().await?;
    /// std::fs::write("captcha.png", session.captcha_image())?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # };
    /// ```
    pub fn captcha_image(&self) -> &[u8] {
        &self.captcha_image
    }

    /// 使用学号、密码和验证码完成登录
    ///
    /// # Arguments
    ///
    /// - `stu_id`: 学号
    /// - `password`: 密码
    /// - `captcha_code`: 验证码，需调用者识别 [captcha_image](CgSession::captcha_image) 后传入
    ///
    /// # Errors
    ///
    /// - [LoginError::CaptchaError] — 验证码错误
    /// - [LoginError::PasswordError] — 密码错误
    /// - [LoginError::LoginFailed] — 其他未知登录失败
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, password)))]
    pub async fn login(
        self,
        stu_id: &str,
        password: &str,
        captcha_code: &str,
    ) -> Result<CgToken, crate::Error<LoginError>> {
        let encrypted_pwd = encrypt_password(password)?;

        let form = [
            ("IndexStyle", "1"),
            ("stid", stu_id),
            ("pwd", &encrypted_pwd),
            ("captchaCode", captcha_code),
        ];

        // 在 self.headers 被 move 之前提取原始 Cookie
        let mut cookies: String = self
            .headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join("; ");

        let res = client
            .post(format!("{}{}", BASE_URL, LOGIN_URL))
            .headers(self.headers)
            .form(&form)
            .send()
            .await
            .network_err()?;

        if res.status() == StatusCode::FOUND {
            let location = res
                .headers()
                .get(LOCATION)
                .ok_or("登录后未获取到跳转地址")
                .unexpected_err()?
                .to_str()
                .unexpected_err()?;

            if let Some(err_code) = location.split("loginErr=").nth(1)
                && err_code != "0"
            {
                return match err_code {
                    "1" => Err(crate::Error::Other(LoginError::PasswordError)),
                    "6" => Err(crate::Error::Other(LoginError::CaptchaError)),
                    _ => Err(crate::Error::Other(LoginError::LoginFailed(
                        err_code.to_string(),
                    ))),
                };
            }

            // 登录成功，合并原有 Cookie 和响应新下发的 Cookie
            let new_cookies = cookie_parser(res.headers().get_all(SET_COOKIE));
            if !new_cookies.is_empty() {
                if !cookies.is_empty() {
                    cookies.push_str("; ");
                }
                cookies.push_str(&new_cookies.join("; "));
            }
            let mut headers = HeaderMap::new();
            if !cookies.is_empty() {
                headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
            }
            return Ok(CgToken { headers });
        }

        let status = res.status();
        #[cfg(feature = "tracing")]
        {
            let body = res.text().await.unwrap_or_default();
            obs::error!(status = %status, body = %body, "unexpected_status");
        }
        Err(format!("登录失败，HTTP {status}")).unexpected_err()
    }
}

/// CG 系统的会话 Cookie，可以通过 [CgSession::login] 获得
#[derive(Debug, Clone)]
pub struct CgToken {
    headers: HeaderMap,
}

impl CgToken {
    /// 从 [HeaderMap] 创建 [CgToken]
    ///
    /// # Arguments
    ///
    /// - `headers`: 一个合法的可用作 [CgToken] 的 [HeaderMap]
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [CgToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }

    /// 获取当前令牌的 [HeaderMap]，可用于 [CgToken::from_headers_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌的 [HeaderMap]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use crate::cg::test::get_cg_token;

    /// 此测试仅验证登录流程能否拿到 token，不检测 token 是否过期。
    /// 若缓存中的 token 已过期，测试仍会通过。
    #[tokio::test]
    #[ignore]
    async fn test_login() -> crate::test::TestResult<()> {
        let token = get_cg_token().await?;
        println!("{token:#?}");
        Ok(())
    }
}
