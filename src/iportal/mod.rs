//! 湖南大学个人门户（iPortal）查询接口。
//!
//! 使用这些接口前，需要先通过 [`login::IPortalToken::acquire_by_cas_login`]
//! 将统一身份认证令牌换取为个人门户令牌。

pub mod card;
pub mod info;
pub mod login;
pub mod personal;
pub mod task;
pub mod term;

mod util;

#[cfg(test)]
mod test;

#[cfg(test)]
mod tests {
    use crate::{iportal::test::get_iportal_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_login() -> TestResult<()> {
        let token = get_iportal_token().await?;
        println!("{token:#?}");
        Ok(())
    }
}
