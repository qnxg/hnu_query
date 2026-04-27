use crate::{
    cas::login::{AccountIssue, CasToken},
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::{client, request::cookie_parser},
};
use reqwest::header::{COOKIE, HeaderMap, SET_COOKIE};
use serde_json::Value;
use std::convert::Infallible;

const GYM_URL_DIRECT_LOGIN: &str =
    "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Login/login";
const GYM_URL_FROM_CAS: &str =
    "http://cas.hnu.edu.cn/application/sso.zf?login=898A822E9695C137E053026B3E0A65D7";

/// 体测系统的令牌
#[derive(Debug, Clone)]
pub struct GymToken {
    headers: HeaderMap,
}

impl GymToken {
    /// 通过统一身份认证系统登录来获得
    ///
    /// # Parameters
    ///
    /// - `cas_token`: 统一身份认证系统的令牌，可以通过 [CasToken::new] 创建
    ///
    /// # Returns
    ///
    /// 返回一个 [GymToken] 实例
    ///
    /// # Errors
    ///
    /// 可能由于用户的账号问题导致登录失败，此时会返回 [AccountIssue] 错误
    pub async fn acquire_by_cas_login(
        cas_token: &mut CasToken,
    ) -> Result<Self, crate::Error<AccountIssue>> {
        let (s_ticket, _) = cas_token.get_sticket(GYM_URL_FROM_CAS).await?;
        // 发送请求
        let _res = client
            .get("http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/view/login/loginPage.html")
            .query(&[("s_ticket", s_ticket.as_str()), ("login_id", cas_token.stu_id())])
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        let res = client
            .post("http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Login/ticketLogin")
            .form(&[("s_ticket", s_ticket.as_str()), ("login_id", cas_token.stu_id())])
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        let cookie = cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ");
        let json_str = res.text().await.unexpected_err()?;
        let json: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;
        if json["info"] != "登录成功" {
            return Err(format!("登录失败: {}", json["info"])).unexpected_err();
        }
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookie.parse().parse_err(&cookie)?);
        Ok(Self { headers })
    }
    /// 直接通过账号密码登录体测系统
    ///
    /// # Parameters
    ///
    /// - `stu_id`: 学号
    /// - `password`: 体测系统密码，一般情况下和个人门户密码相同（也不排除有人专门修改了体测系统的密码）
    ///
    /// # Returns
    ///
    /// 返回一个 [GymToken] 实例
    pub async fn acquire_by_direct_login(
        stu_id: &str,
        password: &str,
    ) -> Result<Self, crate::Error<Infallible>> {
        let res = client
            .post(GYM_URL_DIRECT_LOGIN)
            .form(&[("student_num", stu_id), ("password", password)])
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        let cookies = cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ");
        let json_str = res.text().await.unexpected_err()?;
        let json: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;
        if json["info"] != "登录成功" {
            return Err(format!("登录失败: {}", json["info"])).unexpected_err();
        }
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.parse().parse_err(&cookies)?);
        Ok(Self { headers })
    }
    /// 从 [HeaderMap] 创建 [GymToken]
    ///
    /// # Parameters
    ///
    /// - `headers`: 一个合法的可用作 [GymToken] 的 [HeaderMap]
    ///
    /// # Returns
    ///
    /// 返回一个 [GymToken] 实例
    ///
    /// # Preconditions
    ///
    /// `headers` 参数应该是一个合法的可用作 [GymToken] 的 [HeaderMap]，否则会导致未定义行为
    pub fn from_headers_unchecked(headers: HeaderMap) -> Self {
        Self { headers }
    }
    /// 获取当前令牌的 [HeaderMap]，可用于 [GymToken::from_headers_unchecked]
    ///
    /// # Returns
    ///
    /// 返回当前令牌的 [HeaderMap]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

#[cfg(test)]
mod test {
    use crate::gym::test::{get_gym_token_by_cas_login, get_gym_token_by_direct_login};

    #[tokio::test]
    #[ignore]
    pub async fn test_get_gym_token_by_cas_login() {
        let gym_token = get_gym_token_by_cas_login().await.unwrap();
        println!("{:#?}", gym_token);
    }

    #[tokio::test]
    #[ignore]
    pub async fn test_get_gym_token_by_direct_login() {
        let gym_token = get_gym_token_by_direct_login().await.unwrap();
        println!("{:#?}", gym_token);
    }
}
