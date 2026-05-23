mod raw;

use crate::{
    error::parse_err_with_reason,
    yjsxt::{error::TokenExpired, login::YjsxtToken},
};
use raw::raw_termcode;

/// 根据学年学期获取 termcode
///
/// # Arguments
///
/// - `yjsxt_token`: 研究生系统的令牌
/// - `xn`: 学年，如 `2025`
/// - `xq`: 学期，如 `1`
///
/// # Returns
///
/// 返回对应的 termcode
pub async fn get_termcode(
    yjsxt_token: &YjsxtToken,
    xn: u16,
    xq: u8,
) -> Result<u16, crate::Error<TokenExpired>> {
    let season_name = match xq {
        1 => "秋学期",
        2 => "春学期",
        3 => "暑假学期",
        _ => return Err(parse_err_with_reason("", &format!("无效学期: {xq}"))),
    };
    let target_termname = format!("{}-{}{}", xn, xn + 1, season_name);

    let terms = raw_termcode(yjsxt_token).await?;
    terms
        .iter()
        .find(|t| t.termname == target_termname)
        .and_then(|t| t.termcode.parse::<u16>().ok())
        .ok_or(parse_err_with_reason(
            &format!("{:?}", terms),
            &format!("未找到对应学期: {target_termname}"),
        ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        test::{TEST_XN, TEST_XQ},
        yjsxt::test::get_yjsxt_token,
    };

    #[tokio::test]
    #[ignore]
    async fn test_get_termcode() {
        let yjsxt_token = get_yjsxt_token().await;
        let termcode = get_termcode(&yjsxt_token, *TEST_XN, *TEST_XQ)
            .await
            .unwrap();
        println!("{termcode}");
    }
}
