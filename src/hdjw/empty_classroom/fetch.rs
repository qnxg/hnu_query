use crate::{
    error::{MapNetworkErr, MapUnexpectedErr},
    hdjw::{error::TokenExpired, login::HdjwToken},
    utils::client,
};
use std::collections::HashMap;

const EMPTY_CLASSROOM_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/kbxx/jsjy_query2";

/// 获取原始空教室数据
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌
/// - `xn`: 学年
/// - `xq`: 学期
/// - `week`: 周次
/// - `day`: 星期几，星期一为 `1`，星期日为 `7`
/// - `time`: 节次信息
/// - `building_id`: 楼栋id
pub async fn empty_classroom(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
    week: u8,
    day: u8,
    time: &str,
    building_id: &str,
) -> Result<String, crate::Error<TokenExpired>> {
    let mut form_data = HashMap::new();
    form_data.insert("xnxqh", format!("{}-{}-{}", xn, xn + 1, xq));
    form_data.insert("jxlbh", building_id.to_string());
    form_data.insert("selectZc", week.to_string());
    form_data.insert("selectXq", day.to_string());
    form_data.insert("selectJc", time.to_string());
    form_data.insert("typewhere", "jszq".to_string());
    client
        .post(EMPTY_CLASSROOM_URL)
        .form(&form_data)
        .headers(hdjw_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()
}
