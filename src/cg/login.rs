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
///
/// 使用 AES-128-ECB 模式，密钥为 `Client8Sess!06ID`。
/// 密文经 PKCS7 填充后进行 Base64 编码。
///
/// 不采用密码学安全随机 IV（ECB 模式无 IV），因为这是 CG 系统前端
/// `loginproc.jsp` 页面的 CryptoJS 实现所使用的方法，必须与其保持一致。
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
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
    /// # Arguments
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
    use super::*;
    use crate::cg::test::get_cg_token;

    #[test]
    fn test_encrypt_password_deterministic() -> crate::test::TestResult<()> {
        // 相同的输入应得到相同的结果
        let c1 = encrypt_password("test123")?;
        let c2 = encrypt_password("test123")?;
        assert_eq!(c1, c2);
        Ok(())
    }

    #[test]
    fn test_encrypt_password_output_is_base64() -> crate::test::TestResult<()> {
        let encrypted = encrypt_password("password")?;
        // Base64 只包含字母数字和 +/= 等字符
        assert!(
            encrypted
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        );
        Ok(())
    }

    #[test]
    fn test_encrypt_password_different_inputs_different_outputs() -> crate::test::TestResult<()> {
        let c1 = encrypt_password("password1")?;
        let c2 = encrypt_password("password2")?;
        assert_ne!(c1, c2);
        Ok(())
    }

    #[test]
    fn test_encrypt_password_not_contain_plaintext() -> crate::test::TestResult<()> {
        let encrypted = encrypt_password("mypassword")?;
        assert!(!encrypted.contains("mypassword"));
        Ok(())
    }

    #[test]
    fn test_cg_token_from_headers_unchecked_and_headers() -> crate::test::TestResult<()> {
        use reqwest::header::{COOKIE, HeaderMap, HeaderValue};

        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_static("JSESSIONID=test123"));
        let token = CgToken::from_headers_unchecked(headers.clone());
        assert_eq!(
            token
                .headers()
                .get(COOKIE)
                .expect("COOKIE header should be set")
                .to_str()
                .expect("header value should be valid ASCII"),
            "JSESSIONID=test123"
        );
        Ok(())
    }

    /// 此测试仅验证登录流程能否拿到 token，不检测 token 是否过期。
    /// 若缓存中的 token 已过期，测试仍会通过。如需验证有效性，运行
    /// `test_get_course_list` 或 `test_get_assignment_list`。
    #[tokio::test]
    #[ignore]
    async fn test_login() -> crate::test::TestResult<()> {
        let token = get_cg_token().await?;
        println!("{token:#?}");
        Ok(())
    }
}
