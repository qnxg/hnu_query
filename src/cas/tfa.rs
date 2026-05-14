use crate::cas::login::CasToken;
use crate::error::{MapNetworkErr, MapUnexpectedErr, parse_err_with_reason};
use crate::utils::request::cookie_parser;
use crate::{cas::login::AccountIssue, utils::client};
use regex::RegexBuilder;
use reqwest::StatusCode;
use reqwest::header::{COOKIE, SET_COOKIE};

/// 双因子认证令牌
#[derive(Debug, Clone)]
pub struct TFAToken {
    /// 绑定手机号
    phone: String,
    /// 进行双因子认证时要用到的神秘字段
    execution: String,
    /// 双因子认证时需要提交的 cookie
    cookie: String,
    /// 用于构造新的 CasToken 的学号
    stu_id: String,
    /// 用于构造新的 CasToken 的密码
    password: String,
}

/// 发送短信验证码结果
#[derive(Debug, Clone)]
pub enum SMSResult {
    /// 发送成功
    Success,
    /// 验证码仍在有效期内
    Valid,
    /// 未知错误
    Other(String),
}

/// 双因素认证的结果
#[derive(Debug, Clone)]
pub enum VerifyResult {
    /// 验证通过
    Success(CasToken),
    /// 验证码错误
    CodeError(TFAToken),
    /// 双因子认证令牌过期，
    /// 此时需要再次调用相应的获取令牌的函数，如 `acquire_by_cas_login` 来尝试获取令牌
    Expired,
}

impl TFAToken {
    /// 从双因子认证界面的 html 中创建 [TFAToken]
    pub(super) fn new(
        html: &str,
        cookie: &str,
        stu_id: &str,
        password: &str,
    ) -> Result<Self, crate::Error<AccountIssue>> {
        let regex_execution = RegexBuilder::new(r#"name="execution".*?value="(.*?)""#)
            .dot_matches_new_line(true)
            .build()
            .unwrap();
        let regex_phone = RegexBuilder::new(r#"id="phone".*?name="username".*?value="(.*?)""#)
            .dot_matches_new_line(true)
            .build()
            .unwrap();
        let execution = regex_execution
            .captures(html)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .ok_or(parse_err_with_reason(html, "没有找到execution"))?
            .to_string();
        let phone = regex_phone
            .captures(html)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .ok_or(parse_err_with_reason(html, "没有找到绑定手机号"))?
            .to_string();
        Ok(Self {
            phone,
            execution,
            cookie: cookie.to_string(),
            stu_id: stu_id.to_string(),
            password: password.to_string(),
        })
    }

    /// 获取双因子认证时将要使用的手机号
    pub fn phone(&self) -> &str {
        &self.phone
    }

    /// 发送短信验证码
    pub async fn send_sms(&self) -> Result<SMSResult, crate::Error<AccountIssue>> {
        let res = client
            .get(format!(
                "https://cas.hnu.edu.cn/cas/v2/services/sedsms?mobile={}",
                self.phone,
            ))
            .header(COOKIE, self.cookie.clone())
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?
            .text()
            .await
            .unexpected_err()?;
        match res.as_str() {
            "success" => Ok(SMSResult::Success),
            "valid" => Ok(SMSResult::Valid),
            _ => Ok(SMSResult::Other(res)),
        }
    }

    /// 验证当前双因子认证令牌，调用本函数前需要先调用 [TFAToken::send_sms] 发送验证码
    ///
    /// # Parameters
    ///
    /// - `code`: 验证码
    ///
    /// # Returns
    ///
    /// 验证通过则返回新的 [CasToken]，
    /// 使用新的 [CasToken] 再去申请其他系统的令牌，理论上不会再要求进行双因素认证了。
    ///
    /// 如果验证失败，则会继续返回 [AccountIssue::TFARequired] 错误，需要对新获得的
    /// [TFAToken] 再次调用 [TFAToken::send_sms] 和 [TFAToken::verify]
    pub async fn verify(self, code: &str) -> Result<VerifyResult, crate::Error<AccountIssue>> {
        let res = client
            .post("https://cas.hnu.edu.cn/cas/login")
            .header(COOKIE, self.cookie.clone())
            .form(&[
                ("execution", self.execution),
                ("username", self.phone),
                ("recode", code.to_string()),
                ("reloginType", "reloginPhone".to_string()),
                ("_eventId", "submit".to_string()),
            ])
            .send()
            .await
            .network_err()?
            .error_for_status()
            .unexpected_err()?;
        if res.status() == StatusCode::FOUND {
            // 说明通过了双因子认证，此时这个请求会下发带双因子认证的 cookie
            // 我们和之前的 cookie 合并，就构造出新的 CasToken 了
            let cookies = cookie_parser(res.headers().get_all(SET_COOKIE)).join("; ");
            let cas_token = CasToken::from_cookie_unchecked(
                &format!("{}; {}", self.cookie, cookies),
                &self.stu_id,
                &self.password,
            );
            Ok(VerifyResult::Success(cas_token))
        } else {
            // 除非明确提示验证码错误，否则就认为是令牌过期
            // 双因子认证的界面可能出现这样的情况：如果当前 TFAToken 对应的 cookie 本身
            // 就过期了，那么后面无论怎么输入验证码，都会回到双因子认证界面
            let html = res.text().await.unexpected_err()?;
            if html.contains("验证码错误，请重新输入！") {
                Ok(VerifyResult::CodeError(TFAToken::new(
                    html.as_str(),
                    &self.cookie,
                    &self.stu_id,
                    &self.password,
                )?))
            } else {
                Ok(VerifyResult::Expired)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tfa_token() {
        let html = include_str!("test_data/tfa.html");
        let tfa_token = TFAToken::new(html, "test_cookie", "114514", "1919810").unwrap();
        assert_eq!(tfa_token.phone, "114514");
        assert_eq!(
            tfa_token.execution,
            "3a24971d-2944-4622-818b-77426fe38209_ZXlKaGJHY2lPaUpJVXpVeE1pSjkuYkN0VVVFUXpiVkZvZHk4MmNtWnBhazlIUVd4T2RrODFVbTVOV2pWNU9VSlNRMmRuTjFRemFUUkhjazB4VTNaelQxaHpNMEpHZG5aWldFaGhOVEV4YUV3dk1TczNhbEZUT0ZoS1REUjBlVE54TW5wclpFSmFiMkZOVEhOb1prUkxkazQxV0VabFZ6TXJaREU1TWpKaGFuQjFTbnBpY1ZGdVZFWXZUekZ4T1dsa2NFaGpSRWhDVDJ0NWFVVlBRbEp6Ym5kd1dXNXhlak5LYm5WUFZFUlRhekJ4YkhONFYzSk5WRWc1WVZCaGFrSk9hazVxVm5KTFkyVTJNbTF2TXpsbWRYTjVhSGMxTXk5WFQySTNSVU5oYkVsQmFXTXdMMFJLYVVOU1JEWnlPWHBRV1VreWFtMU9hak5uWjIxdWFHVk9MMDVWVVdoWU1qSlFOakJqTkVGWWREQlZlREpwZEhkVlYzWkJiblUzZGpSV0wybFRaRWRzTnpCSGEzZ3pVRkJLUTFCVVlYSktaRFJUVkZCUFV6Unlja1JVWVdkMWRGZzRPWGhKVG1kRFVqRk5PVlJzV1djdlVHSnVaMlZhU1VsclJsWjFaRU1yUVhoME0zazBibFpsUjBKVWJXcEVVVVZ5UkhweVpGVkNXVmxSU1dFdlZuWTJhQzltUVVoWWQwUTJUbloyVHk5UGRtbHdXbGN5VlhwdldFcHRZVlp5TDJoM1JIbEhNalJhVUdVNFZuUXpkMVZ4TkZkb2NuZG1ja2s0ZEZCUVNXMHpTWGxCYm5aVVlVbHRXV3BvV0VKbk5VWjNPVXAzWms1NmJYaGlUbWRpVW1ORFMxVnZURFIzYXpSM2VWTktjbXh5YW5FMGJHTm5SMjF5VVhSbFVFRTFNMnBYUmxSUVJHUXpUVXgyWlhkbVMwUjVZMHhpTUdkVVdXWmlhbk5wVXpKV1UwTTFlbWxoV2k5UFFXTlRXblZHWXpSWVFYazVUbTFoUWl0c0syWnZSa1Y0ZFVWbVZtdGlhMUZIV0d4dFZpODFTMk5NVGxBdlF6RkxhRTA1VkhCbGFUZzVNbXhZVW1JME4yTnpORkE0TldZNVZtUTJTRXczWWxweVVYaG5RMjE2UVRKamFUSklLM2hZTVVsa1QyRTFNVnB4YWxSMWRqSnRiV0ZhUkZSeGVWZFNZVnB0VUhRNVpEVXpkMFZWTlc1VGFuTnlaVVk1V0VwVlVURnpkbTUwWWxsSGVGTXpWbGRGVTFka2R6VkxkVFoxZFdFNGQyUTRjbHAxYXpGTGJYQm9RbUZxVmpNMVJ6RlBNVWhhUVhkNU1rcFROMEV4YkhaS09VVk1kME5MV1hwaVVHUndXR0pSU2sxbGR6UTFWbXBtY1ZWeGMxRm5RMFpFU1VkRVVYWTFRblkyVEZBd04ycERZbTE2ZWxCVWMwWXJTWFF4VjNsWVQxWlNkVGhwUWxOdE5YRjJVa05hVVVwTVVtTkRhbXRRYTBVd1VESkZSRTVhTW5Cc1pGQXlZbkJxZDJwUVJGSlFNRVJQWXpNMksxTjRORFpXTkhSWVkycGFOMk5VYVZKWk1rbHZibGRRY1M5d1R5dDNTWHB1Y1dKMWRIbHdSR2RZVjFnMU9EZG9TV1JEWTA1b2MwNVdjRlY2TW1SclFtOVNNMDRyVlZSalV6VnNaVzA0Wm04d1VETkVlWFZtVjBGM1FWUkxlVUZhUjJOMUwyMTNaRGRRZFc0clVsVlJVR05rSzBkWWJEUnRSSE5RZFd4WlJHNVlMelpFVmtwVFNXcDZSalpWT1hSTmJIVkdabGgxY1RsTFNscHdiWEJ1ZGpsdFRXRlFNeXRtV1ZZM1NpdHlhelkzYkhWeEszTlhkRkl3UjFreVNHZHlUa042WWxCT1RWWkZOVFJhYTA0ME1HbFlRWHBtYUZWTGRFVjNSR1pTYkV4SFJrdFRURXhIU2lzeFVUSk5ha3RRYWxoWGJYcEdSbnA2UmpCbVFVSmpUSEExU1N0TGJGQXdZbWxOTXpnMGJVaGtMM05ZZVZaek1EUnhRa1ZwVDBWcVNFOHdNM1JuZVhKVWJEaFhVRVp0VjNkQ1VrVmxNR2xyTURFeFREQm1kbU5ST0VkR2JUVlRkV3BGU0hWM2FVSk1PVUl3YWpKMlRtYzRSR1JxYUhoYVlrdEVTQ3RxUmtOTWFXTldXVVpoUzB0a1Z6SnBUbEl5VFVsb1ZpOVlkRWxKYmxsVVUyUjJUREZRYWtScFRUQnhjMk4zTjJkcU5GVkVXVnBaSzBSRWNtUXJabFZxY1ZOcGEySm1PRUZQY0d0QlZYSnNXbVl6T0RoSFREQm1TSFpaZFcxMk0zbDRhVFozUnpKSlRWbGxSbXBYUm00emJYcFFRV2ROTlRKUlZWUkhXRXg0WjNWdE1UY3hSSE5RTldJNGMya3JRMk5CVkdFeFNHeE9UbTFhTjJFeVVFdDZOWEpDY0ZKQ1RFcEdTVGc1Y3l0UmRVNDBMMHhSTkVNMmFrVnBjbWhyZDBreVVWSmxNRFJHWTNGWVVUZEplWEJyTUZvMlVtSnFZWFZ5ZEV4dFpUVlVXVXgwVVRWRGMyZE1XVkpXVkRaR2VteFpkMHhIWWpGbmNtRXZXWGRTVGxSdFJITm9NelpRTmxwa2VFSkdjVWRwZEVSbFJGVmtURE4xSzB0WE5ESkxNbkprT0VsNVZXZDRORmxOZEU1elVVczVhSE5PT1ZWeE5qSlBPRGRtU1VocVNVVjRTbUl4VkRRdlNXUkxRemwyU1U1M1YzUkhRakprUWs5SFZUZ3pjWEpMWW5aa1dERlhTSFZyUWxONE9GQjZSRGh4TjBRck0yWXJkRmRXU2toc056RlFXREI2ZUZwR01XZEVVR3RZUWxGVFNqZEJOakkxYkdseFoxUTRWVWhKTUZKTldISmhlbFJuVGpWUmFHUldRV2xGWjI1ck1ucHRXR055VlhsRE9FSm1WR0pTY3pKUU1YSlpiSFpOVFZwalZraFBhR3RLVW5JemRFSnlTSEIzVTNCR1dWbFRSRXBXZVRsd2VXdHliV1JvUjFwR1kzUjVZVzVpVUZsSVRucGtSMVF6TW5sa01HaDJUVGRVTWpOVVlpdGpURWR6VDJ3d1lucHNWWEptTkdvd1RIcFVXRFJzUzBoeE1taEZiRVZuWXk5aGJFUldWR1JMU2pKQ2NqSXdSWFZaUkZWeVRVWlhNRlZXY0VSTloybG1RVFo1YUdoTE9YQmFkR2xDVDFJeWFsTjFZMHhIUW01T2RrUnJTRTFLWTJ4cmMxVlJhRWhWZDJKaVJrZEJNRnA1VGtodGRFTm1UV1JwU1VaNGNXRXpTRk55ZFRKVFpURmxLMlJTZERCamJuUlZXR2xGWlVJM1NVUktZWEZMYWtrNVJYQlRORlpuYzFGdlVubzNObEZsVFRWdWVVNUJhMGRsU0V0M1RVSjJVVEowZHpac1VXWm1NR3BZUmxnME5ISmtNRWd3U2t4S1ZFeDJObmxrY1VwTlYybEZVbU5TTlc5WE9ESmhPV3BVY0VKcWIxZEtRblEwWVZSRVNUZEdTWHBwT1hWMU9UZHZVRFF6TTNFeU1rMXlRMnBGVW01UFMzcFNWVzFJS3pWSlJVeHhOVlZKTW5BclYyTlROek4xTDJNNU1uSmxXa1Z6U2psc1ZUZGhkbTlVWmtoUlZtNVJaekp6WVhaNllUbGtTa0U0Y1hkUFlYSm1SbXQwYUhNM1RuWkRSbVJEV0ZwMlVXZEVha0pHUm01aFVHRkxRMEZSTW1vNWFGZE5iMU52YUdWT1VreGlOREZqYTA5NFNIZzBSMmhUTmxwelNsWk5kRlpGV0M4MVFtVjRUR1poYWl0eFVucE5RbVpzUnpOTFNsSTBRMnh0YnpCVE0zVXdhR012ZW1oeWJDOHdPRTF2VEZZd1dIcHhTMHBhYmpoSGNHcGpkMXA2UW1sTVEwdE9iSFZITUZnMVZFc3hRMDFpU0RsVU9YTm1SbVpLZVVoTFRuaHJNMlpaWjBaR1RWQlVNMFpYYlROWGVqbElXRkZNZFVNNVluWjVTVU41U1dKMFJUaGFkVVpqY1Roc1ltSTVOMHd6WlZSbWRuVlJkbVkyY1VOYWNHRlRVVUVyVVhSR00zWkxiazVHUm1nM2NXNU5RbGt2YzNoUFppOXNRM1l6UkRZMGJYbDZOa04zVEdSeE1XMVRUVWh1YzJoMlZFNTBjMGxRVUUxRFIwdzNhVk42WkZOUGIwOUtjRGxvU0Vrd0syWk5jMDFOZVZZNFZVcGxiMWRRUTBoMFlWTldUVWgyT1VJcllrSjFVakJvTjBsNVZtVXJjbWMyTlVwQlkycHBRbE5KV0UwMllUbFZhRko2VURscFMyVnZPWFpOYUVad2VFaHpUMFZ6VUN0WVdtaFVSV1owVDBvNWJEUTVPRE5YTldGaVVXa3dhME5hV1ZFMWExVlpWWE5DWm1vclRqVmpMMmxDV1VsS09FVkhRMU5YWmtzMmRVUmlXR3RZUkRSb2FHcFVjM1k1VVVKTmMwTXlWazVCZGxac1kyODBhVTlvVGpRNUswcEZTV1ZaZVhGNFJ6RjBVbGxaWW1OTGIybFhVbWRKVTBsNlNFUkpkM0ZLTURaa1FVdFRNRGxNU0ZsRVNYaEVLMlZPUzFSVWEwVXdiV0ZQTkdaclJVcHlTekJzTVZWV2RHMU5aRkpOZGxwdGJXeFJOVlkxU0RWVlMzVjNWMDF2TjBkcFdUTTRUbmhQTVdGb2N6UjVRM0l5TW5nMmNqZ3ZRVkpNUTJ4dFZtVjVURFV4Wms1eFJIbHNRMGxqZUVkYVFUTnJTM1F6ZDFCM2FWTjNZM05WVTBSeVYwVjJTSGhQUkhReWJtRlNNWHBFT1ZGalpXWndTVFJZVmpaR1UwdDJNRXhMYWxGbU9WSTFiV2t2WldodlRuRmpZVU40VW1waWRFRmpabnBoY25sMldteE1NblU0V0ZoTFUxZHVRMXBPVVRCQ1owcGhTM2h2YVVNd1duSlJUbkZYUlZKMGFtNWxVR0psYldSM09EWjRiVU5yUmsxSGMzZDVkRzU2Vkd4V2JrWldhRVl6UzFBcmIxbzNRME50TUdsUlpubEtWRVZaUkRsaGNsVkdLMVZqU2xGTmJreEdjbVZqVG5Jd1RXcE1lRXBsVUc1RlQybHVlRXRZVHpoTVIyeEdTWHBuWkRnd2NGQXZaMHh3Y0hWUk5tNTJUMDlRWldObWMweFJVa0pIWkRNclIzcE5Oa1Z2TWpkMGNGTlJXbFkxWlN0c1NWbFVNVTVSYTNCUVJHaE9jbGRETXpOR09VMUZNVGhYWXpoblVWZG9RVlpuYW01RmFYaFVRVk50VmtsbVVXSkNNbWxoVlRKeVNXVXhVR0p5TTFOYWMwUnhOamRrUWpWSFIyRkRXV0V3VTNSSFNGTlZjalJ1WVVSdmRqWkJWbkZKVW1FMVpXaFdjR0l4VjJOU2FqSTViWEk1WkdNNVlreHFlbXB4VDFnelZXVk5WVzFDVlZacFRuTkdjR0YwZFVnd1YwZFBTRnBtVTBGNVZtbHBVV00xYVhSTFZESm1lVVV6VFRkTWNFUnhkMjVoZEZsNlJuQXlhRVF5YlZsR016UXJiRGQ2Y1RaaGRUUXdZV1JNU3pGNVQxQjRPVGswUkV0bk5tTlNTbkZqZWxacmJuaERjbmhRZDBKd1NGaFhlWGwwUWxReVVGVkdkbXRFTWpNeVR6SmljbXhoVEU5d1EzUk5WbTFXYjNWa2NrMUdhVVkyVTJsQlJEQjNNM1k0UkhaWllVSkpaRUp1TW01RGQxVTJVV2w0UkVKRE4zRk9jRUoxZUhkRFkxaG5kVkJ1VjBsWVVsb3JXRlpDZEhjMFMxcEdNRGxDYTI5NlQzRkRhbXBxWkV4dFNreENPR2hsYVhnMmRtSjZNR1JCVURrNVVrdzVkV2hhZGsxa1VuVmhZV0pPYldGRGFVSnRiSEZNU0RCUmVYRlJZVVpHUjA1UVdVNXBSa1JyWmxoeE56TnhPVFZQTkhKbmJGa3dWbEZPUkVwR2FqUkZOMHRPVGxZMFdIY3pXVWRKVkdWRllWUnVhekpYUnpJeFZGSnpiMDUwVlhaVWJWRnZOak13WVZSV1JFSnVWMDlsYXpSMlpVcFhPVEV3TmxOM2RITlNRM1UyYzJObE1rWkNjMnRVWW5Zd2NYSnJVVzlzU1VsclRsTmlNRGswWkhwSVNrdERUQzlaVGtoMmFETmpOSGxWWTFwV00xSkVRVmxMVUZGb1ZEaHBNVmxZVDFGclREaEZaaXRWY0V3cmJuQnFhbXBOVlVwT2VGbDNNR053U0VScVdrTjBNV2sxV2xacVptUlVUR3RUWW5CcGFqWXhkRmxEU1VWamNpOTJRVVJYV21keVRFeGtXR3cyU0dWeWVXeG1NVzlUZWpWMGNWWlliamhqVVd0RlJreGxRbnBpZFVGTVJuSm1OSFp4ZUdoVVVFcHRaVmh5Y1dSeWNWZG5kbWRwVjFKYU4wRjRlSE50V1VwR2MycGxjblZUVVRBeVIzRlJTV1V4U2t0M1JIVnhTRE5FS3poTGIyczNaVEJFUVN0SGNRLlFyS05qSVRjZ01hcGNUbkM0XzFKSFdrTEE3VzVVRzJ2OWJKQTZMYl9CVXFDckZ3aGxjdWprWF9WWk1wWHZrRlV1bDkwejd0WGhYN3kwZHNTdHRzU0N3"
        );
    }
}
