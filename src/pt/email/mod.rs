mod raw;

use crate::pt::{email::raw::raw_unread_email_data, login::PtToken};
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
pub async fn get_unread_email_count(
    pt_token: &PtToken,
) -> Result<Option<u32>, crate::Error<Infallible>> {
    let res = raw_unread_email_data(pt_token).await?;
    Ok(res.unReadCount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pt::test::get_pt_token;

    #[tokio::test]
    #[ignore]
    async fn test_get_unread_email_count() {
        let token = get_pt_token().await.unwrap();
        let unread_email_count = get_unread_email_count(&token).await.unwrap();
        println!("{:#?}", unread_email_count);
    }
}
