use crate::{
    cg::error::LoginError,
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, request::cookie_parser},
};
use aes::cipher::{BlockEncryptMut, KeyInit, block_padding::Pkcs7};
use base64::engine::{Engine, general_purpose::STANDARD as base64};
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};

const BASE_URL: &str = "https://cg.hnu.edu.cn";
const LOGIN_PAGE: &str = "/indexcs/simple.jsp";
const CAPTCHA_URL: &str = "/cgjiaoyan";
const LOGIN_URL: &str = "/login/loginproc.jsp";
const AES_KEY: &str = "Client8Sess!06ID";

type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;

/// 对密码进行 AES-ECB/PKCS7 加密（与前端 CryptoJS 保持一致）
fn encrypt_password(password: &str) -> Result<String, crate::Error<LoginError>> {
    let key = <aes::cipher::generic_array::GenericArray<u8, _>>::from_slice(AES_KEY.as_bytes());
    let cipher = Aes128EcbEnc::new(key);
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(password.as_bytes());
    Ok(base64.encode(&ciphertext))
}

/// CG 系统的登录会话
///
/// 创建一个与服务器绑定的 session（包含 JSESSIONID），并下载验证码图片。
/// 用户识别验证码后，调用 [CgSession::login] 完成登录。
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
    pub async fn new() -> Result<Self, crate::Error<LoginError>> {
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
            .network_err()?
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
    /// # use hnu_query::cg::CgSession;
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
    /// # Parameters
    ///
    /// - `stu_id`: 学号
    /// - `password`: 明文密码（内部使用 AES-ECB 加密）
    /// - `captcha_code`: 验证码，需调用者识别 [captcha_image](CgSession::captcha_image) 后传入
    ///
    /// # Errors
    ///
    /// - [LoginError::CaptchaError] — 验证码错误
    /// - [LoginError::PasswordError] — 密码错误
    /// - [LoginError::LoginFailed] — 其他未知登录失败
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
        let original_cookies: Vec<String> = self
            .headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|s| s.split("; ").map(String::from))
            .collect();

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
                    _ => Err(crate::Error::Other(LoginError::LoginFailed(format!(
                        "未知错误码: {err_code}"
                    )))),
                };
            }

            // 登录成功，合并原有 Cookie 和响应新下发的 Cookie
            let mut cookies = original_cookies;
            cookies.extend(cookie_parser(res.headers().get_all(SET_COOKIE)));
            let cookies = cookies.join("; ");
            let mut headers = HeaderMap::new();
            if !cookies.is_empty() {
                headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
            }
            return Ok(CgToken { headers });
        }

        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        Err(format!(
            "登录失败，HTTP {status}: {}",
            &body[..body.len().min(200)]
        ))
        .unexpected_err()
    }
}

/// CG 系统（计算机学院课程教学辅助系统）的令牌
///
/// 存储登录后的会话 Cookie，用于后续 API 请求。
#[derive(Debug, Clone)]
pub struct CgToken {
    headers: HeaderMap,
}

impl CgToken {
    /// 使用学号、密码和验证码一步完成登录（便捷方法）
    ///
    /// 当你已经知道验证码时可以使用此方法。如果需要查看验证码图片，
    /// 请使用 [CgSession::new] 获取会话和验证码图片，再调用 [CgSession::login]。
    ///
    /// # Parameters
    ///
    /// - `stu_id`: 学号
    /// - `password`: 明文密码（内部使用 AES-ECB 加密）
    /// - `captcha_code`: 验证码
    pub async fn login(
        stu_id: &str,
        password: &str,
        captcha_code: &str,
    ) -> Result<Self, crate::Error<LoginError>> {
        CgSession::new()
            .await?
            .login(stu_id, password, captcha_code)
            .await
    }

    /// 从 [HeaderMap] 创建 [CgToken]，用于缓存恢复
    ///
    /// # Preconditions
    ///
    /// `headers` 应该是一个合法有效的 [CgToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }

    /// 获取当前令牌的 [HeaderMap]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use crate::cg::test::get_cg_token;

    #[tokio::test]
    #[ignore]
    async fn test_login() -> crate::test::TestResult<()> {
        let token = get_cg_token().await?;
        println!("{token:#?}");
        Ok(())
    }
}
