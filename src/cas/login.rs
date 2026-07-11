use crate::{
    cas::{error::TokenExpired, tfa::TFAToken, utils},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::{client, request::cookie_parser},
};
use regex::Regex;
use reqwest::{
    StatusCode,
    header::{COOKIE, LOCATION, SET_COOKIE},
};
use serde_json::Value;
use std::sync::LazyLock;

const PUBKEY_URL: &str = "http://cas.hnu.edu.cn/cas/v2/getPubKey";
// 这个是sso.zf跳转用到的一个链接
const SERVICE_URL: &str =
    "http://cas.hnu.edu.cn/cas/login?service=http://cas.hnu.edu.cn/system/login/login.zf";

/// 统一身份认证系统的令牌
#[derive(Debug, Clone)]
pub struct CasToken {
    cookie: String,
    stu_id: String,
}

/// 用户账号问题导致登录失败的错误
#[derive(thiserror::Error, Debug, Clone)]
pub enum AccountIssue {
    /// 密码错误
    #[error("密码错误")]
    PasswordError,
    /// 密码应该修改
    ///
    /// 可能是信息化办将密码进行重置了，必须前往个人门户修改密码后才能登录
    #[error("请前往个人门户修改密码后重试")]
    PasswordShouldChange,
    /// 账号因多次输错密码被锁定
    #[error("账号因多次输错密码被锁定")]
    AccountLocked,
    /// 需要双因子认证，输入手机号后获取验证码
    #[error("需要双因子认证")]
    TFARequired(TFAToken),
}

impl CasToken {
    /// 获取当前令牌对应的 Cookie，可用于 [CasToken::from_cookie_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌对应的 Cookie
    pub fn cookie(&self) -> &str {
        &self.cookie
    }
    /// 获取当前令牌对应的学号，可用于 [CasToken::from_cookie_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌对应的学号
    pub fn stu_id(&self) -> &str {
        &self.stu_id
    }
    /// 从 Cookie 创建令牌
    ///
    /// # Arguments
    ///
    /// - `cookie`: 一个合法的可用作 [CasToken] 的 Cookie 字符串
    /// - `stu_id`: 该 `cookie` 对应的学号
    ///
    /// # Returns
    ///
    /// 返回一个 [CasToken] 实例
    ///
    /// # Preconditions
    ///
    /// - `cookie` 参数应该是一个合法的可用作 [CasToken] 的 Cookie 字符串，否则会导致未定义行为
    /// - `stu_id` 参数应该和 `cookie` 对应，否则会导致未定义行为
    pub fn from_cookie_unchecked(cookie: &str, stu_id: &str) -> Self {
        Self {
            cookie: cookie.to_string(),
            stu_id: stu_id.to_string(),
        }
    }
    /// 通过学号和密码登录统一身份认证系统，获取 [CasToken]
    ///
    /// # Arguments
    ///
    /// - `stu_id`: 学号
    /// - `password`: 密码
    ///
    /// # Returns
    ///
    /// 返回一个 [CasToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题导致登录失败，此时会返回 [AccountIssue] 错误
    pub async fn acquire_by_login(
        stu_id: &str,
        password: &str,
    ) -> Result<Self, crate::Error<AccountIssue>> {
        let res = client
            .get(SERVICE_URL)
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        if res.status() != StatusCode::OK {
            return Err("响应的状态码异常，应为OK").unexpected_err();
        }
        let mut cookies = cookie_parser(res.headers().get_all(SET_COOKIE));
        // 拿到登录表单的execution和_eventId
        let login_text = res.text().await.unexpected_err()?;
        static EXECUTION_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r#"name="execution" value="(.*?)""#)
                .unwrap_or_else(|e| panic!("invalid EXECUTION_RE regex: {:?}", e))
        });
        static EVENT_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r#"name="_eventId" value="(.*?)""#)
                .unwrap_or_else(|e| panic!("invalid EVENT_ID_RE regex: {:?}", e))
        });
        let execution = EXECUTION_RE
            .captures(&login_text)
            .and_then(|cap| cap.get(1))
            .map_or("", |m| m.as_str())
            .to_string();
        let event_id = EVENT_ID_RE
            .captures(&login_text)
            .and_then(|cap| cap.get(1))
            .map_or("", |m| m.as_str())
            .to_string();
        // 通过pubkey接口获取modulus和exponent
        let pubkey_res = client
            .get(PUBKEY_URL)
            .header(COOKIE, &cookies.join("; "))
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        // 获取pubkey的cookies
        cookies.extend(cookie_parser(pubkey_res.headers().get_all(SET_COOKIE)));
        let pubkey_str = pubkey_res.text().await.unexpected_err()?;
        let pubkey: Value = serde_json::from_str(&pubkey_str).parse_err(&pubkey_str)?;
        let modulus = pubkey["modulus"]
            .as_str()
            .ok_or_else(|| parse_err(&pubkey_str))?
            .to_string();
        let exponent = pubkey["exponent"]
            .as_str()
            .ok_or_else(|| parse_err(&pubkey_str))?
            .to_string();
        // 加密密码
        let encrypted_password = utils::rsa_encrypt(password, &exponent, &modulus)
            .ok_or("RSA encryption failed")
            .unexpected_err()?;
        // Post登录表单
        let login = client
            .post(SERVICE_URL)
            // .header(CONTENT_TYPE, "application/x-www-form-urlencoded")   //
            // 这个header会自动加上，不用手动加
            .header(COOKIE, &cookies.join("; "))
            .form(&[
                ("authcode", ""),
                ("username", stu_id),
                ("password", &encrypted_password),
                ("execution", &execution),
                ("_eventId", &event_id),
            ])
            .send()
            .await
            .network_err()?;
        if login.status() == StatusCode::FORBIDDEN {
            return Err(crate::Error::Other(AccountIssue::AccountLocked));
        }
        // login_params里面的pv0在后面的请求也会有用(netflow)
        let addition: Vec<String> = cookies
            .into_iter()
            .filter(|cookie| cookie.starts_with("_pv0="))
            .collect(); // 错误已在前面被处理，一定会有_pv0
        let mut cookies = cookie_parser(login.headers().get_all(SET_COOKIE));
        cookies.extend(addition);
        // 先获取 location，不然后面的 .text() 会消耗掉 login
        let location = login.headers().get(LOCATION).cloned();
        let login_result = login.text().await.unexpected_err()?;
        if login_result.contains("用户名或密码错误") {
            return Err(crate::Error::Other(AccountIssue::PasswordError));
        }
        if login_result.contains("双因子认证") {
            let tfa_token = TFAToken::new(&login_result, &cookies.join("; "), stu_id, password)?;
            return Err(crate::Error::Other(AccountIssue::TFARequired(tfa_token)));
        }
        let location = location
            .ok_or_else(|| format!("无法获取 cas 响应的 location: {}", login_result))
            .unexpected_err()?
            .to_str()
            .unexpected_err()?
            .to_string();
        const PASSWORD_SHOULD_CHANGE_PAT: &str = "cas.hnu.edu.cn/securitycenter/modifyPwd/index.zf";
        if location.contains(PASSWORD_SHOULD_CHANGE_PAT) {
            return Err(crate::Error::Other(AccountIssue::PasswordShouldChange));
        }
        Ok(Self {
            cookie: cookies.join("; "),
            stu_id: stu_id.to_string(),
        })
    }
}

impl CasToken {
    /// 通过统一身份认证系统登录对应的平台，获取回调链接（即 `ticket_url`）
    ///
    /// # Arguments
    ///
    /// - `service_url`: 登录服务地址
    ///
    /// # Returns
    ///
    /// 返回回调链接 `ticket_url`
    ///
    /// # Errors
    ///
    /// 可能由于当前 [CasToken] 过期，此时会返回 [TokenExpired] 错误
    pub(crate) async fn get_ticket_url(
        &self,
        service_url: &str,
    ) -> Result<String, crate::Error<TokenExpired>> {
        let Ok(res) = client
            .get(service_url)
            .header(COOKIE, &self.cookie)
            .send()
            .await
            .network_err()?
            // 状态码为 4xx 和 5xx，都保守地认为是令牌过期了
            // 后续可能还要再验证一下有没有必要这么保守
            .error_for_status()
        else {
            return Err(crate::Error::Other(TokenExpired));
        };
        if res.status() == StatusCode::OK {
            return Err(crate::Error::Other(TokenExpired));
        }
        if res.status() != StatusCode::FOUND {
            return Err(format!("获取ticket_url时失败，HTTP代码 {}", res.status()))
                .unexpected_err();
        }
        let ticket_url = res
            .headers()
            .get(LOCATION)
            .ok_or("没有在 location 中找到 ticket_url")
            .unexpected_err()?
            .to_str()
            .unexpected_err()?;
        Ok(ticket_url.to_string())
    }
    /// 登录形如 <http://cas.hnu.edu.cn/application/sso.zf?login=B5712DC2FA281C96E053026B3E0A80A6>
    /// 这样的链接的服务
    ///
    /// # Arguments
    ///
    /// - `service_url`: 登录服务地址
    ///
    /// # Returns
    ///
    /// 返回一个 (s_ticket, cookies)，用于后续操作
    ///
    /// # Notes
    ///
    /// 在最开始发现校园网流量系统的登录逻辑不能继续沿用 [CasToken::get_ticket_url]，
    /// 而是需要用到这里这个函数的逻辑来进行登录，
    /// 见 <https://code.qnxg.cn/qnxg/spider_2024/pulls/1>
    ///
    /// 后来发现体测系统也需要这么做了，于是就把这个逻辑抽取出来放在这里了，所以这个
    /// 函数本身的实际意义并不明显
    pub(crate) async fn get_sticket(
        &self,
        service_url: &str,
    ) -> Result<(String, String), crate::Error<TokenExpired>> {
        // 后面可能会进行多次重定向才能拿到 s_ticket，由于目前 client
        // 关闭了跟随重定向，所以我们手动模拟
        let mut now_url = service_url.to_string();
        let mut cookies = self.cookie.clone();
        let mut s_ticket = None;
        // 分析的是大概重定向 4 次就可以拿到 s_ticket，为了保险起见多循环几次（中间拿到 s_ticket
        // 就会 break）
        //
        // 这里的重定向次数似乎是因人而异的，原因不明
        for _ in 0..6 {
            if now_url.starts_with("https://cas.hnu.edu.cn/sprcialapp/zf_form/index.zf") {
                s_ticket = Some(
                    now_url
                        .split('&')
                        .find(|s| s.starts_with("s_ticket="))
                        .and_then(|s| s.split('=').nth(1))
                        .ok_or("malformed s_ticket parameter")
                        .unexpected_err()?,
                );
                break;
            }
            let res = client
                .get(&now_url)
                .header(COOKIE, &cookies)
                .send()
                .await
                .network_err()?
                .error_for_status()
                .unexpected_err()?;
            if res.status() == StatusCode::OK {
                return Err(crate::Error::Other(TokenExpired));
            }
            if res.status() != StatusCode::FOUND {
                return Err(format!(
                    "获取s_ticket时失败，HTTP代码: {}, url: {}",
                    res.status(),
                    now_url
                ))
                .unexpected_err();
            }
            now_url = res
                .headers()
                .get(LOCATION)
                .ok_or("获取重定向链接失败")
                .unexpected_err()?
                .to_str()
                .unexpected_err()?
                .to_string();
            cookies = format!(
                "{}; {}",
                cookies,
                cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ")
            );
        }
        let s_ticket = s_ticket
            .ok_or("获取s_ticket失败，未找到s_ticket")
            .unexpected_err()?
            .to_string();
        Ok((s_ticket, cookies))
    }
}
