use crate::{
    cas::{
        login::{AccountIssue, CasToken},
        tfa::VerifyResult,
    },
    error::MapUnexpectedErr,
    test::{TEST_CAS_CACHE, TEST_PASSWORD, TEST_STU_ID},
};
use std::io::{Read, Write};

/// 创建一个测试用的 [CasToken]
///
/// 如果设置了 `TEST_CAS_CACHE`（详见 `docs/test.md`）
/// 这个函数会尝试从工作目录下的 `cache` 文件夹中加载之前已经缓存的 CasToken
///
/// 如果没有设置 `TEST_CAS_CACHE`，或是没有从 `cache` 中找到已经缓存的 CasToken，
/// 则登录账号，获取 CasToken，并以交互的方式来处理需要双因子认证的情况。
///
/// 通过登录获取了新的 CasToken 之后，如果设置了 `TEST_CAS_CACHE`，则会将
/// 新的 CasToken 自动缓存到 `cache` 文件夹中。
pub async fn get_cas_token() -> Result<CasToken, crate::Error<AccountIssue>> {
    let stu_id = TEST_STU_ID;
    let password = TEST_PASSWORD;
    let cache_name = format!("{:x}", md5::compute(format!("{}{}", stu_id, password)));
    if *TEST_CAS_CACHE {
        println!("使用 CasToken 缓存: {}", cache_name);
        let mut cache_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("cache/{}", cache_name))
            .unexpected_err()?;
        let mut cookies = String::new();
        cache_file.read_to_string(&mut cookies).unexpected_err()?;
        if !cookies.is_empty() {
            return Ok(CasToken::from_cookie_unchecked(&cookies, stu_id));
        }
    }

    let result = CasToken::acquire_by_login(stu_id, password).await;
    let cas_token;
    match result {
        Ok(v) => {
            cas_token = v;
        }
        Err(crate::Error::Other(AccountIssue::TFARequired(tfa_token))) => {
            let mut tfa_token = tfa_token;
            // 测试时，要求手动输入验证码
            loop {
                print!("需要双因子认证({}), 是否继续(y/n): ", tfa_token.phone());
                std::io::stdout().flush().unexpected_err()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unexpected_err()?;
                if input.trim().to_lowercase() == "y" {
                    break;
                } else if input.trim().to_lowercase() == "n" {
                    panic!("测试停止");
                }
            }

            loop {
                let res = tfa_token.send_sms().await?;
                println!("发送验证码结果: {:?}", res);
                print!("请输入验证码（输入 -1 重新发送验证码）: ");
                std::io::stdout().flush().unexpected_err()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unexpected_err()?;
                let input = input
                    .trim()
                    .parse::<i32>()
                    .map_err(|e| format!("invalid verification code: {e}"))
                    .unexpected_err()?;
                if input == -1 {
                    continue;
                }
                let verify_result = tfa_token.verify(&input.to_string()).await?;
                match verify_result {
                    VerifyResult::Success(token) => {
                        cas_token = token;
                        // 跳出最外层 loop，然后就会再次循环，相当于重试了
                        break;
                    }
                    VerifyResult::CodeError(new_tfa_token) => {
                        println!("验证码错误，请重新输入");
                        tfa_token = new_tfa_token;
                    }
                    VerifyResult::Expired => {
                        panic!("双因子认证令牌过期");
                    }
                }
            }
        }
        Err(e) => return Err(e),
    }
    if *TEST_CAS_CACHE {
        let mut cache_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("cache/{}", cache_name))
            .unexpected_err()?;
        cache_file
            .write_all(cas_token.cookie().as_bytes())
            .unexpected_err()?;
    }
    Ok(cas_token)
}
