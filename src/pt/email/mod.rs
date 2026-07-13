mod fetch;
mod parse;

use crate::pt::login::PtToken;
use std::convert::Infallible;

/// 获取未读邮件数
///
/// # Arguments
///
/// - `pt_token`: 个人门户令牌，可以通过 [PtToken::acquire_by_cas_login] 获取
///
/// # Returns
///
/// 未读邮件数
///
/// 如果返回 None，说明未绑定邮箱，需要前往个人门户 -> 安全中心绑定邮箱
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(pt_token), fields(subsystem = "pt"), err)
)]
pub async fn get_unread_email_count(
    pt_token: &PtToken,
) -> Result<Option<u32>, crate::Error<Infallible>> {
    let json_str = fetch::unread_email_count(pt_token).await?;
    parse::email_unread_count(&json_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{pt::test::get_pt_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_unread_email_count() -> TestResult<()> {
        let token = get_pt_token().await?;
        let unread_email_count = get_unread_email_count(&token).await?;
        println!("{:#?}", unread_email_count);
        Ok(())
    }
}
