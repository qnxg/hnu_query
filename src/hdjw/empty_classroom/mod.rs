mod parse;
mod raw;

use crate::hdjw::{error::TokenExpired, login::HdjwToken};
use serde::{Deserialize, Serialize};

/// 空教室信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmptyClassroom {
    /// 教室名称，如 `综105`
    pub room_name: String,
    /// 教室类型
    // TODO 添加示例
    pub room_type: String,
    /// 座位数
    pub seat_count: u32,
    /// 考试座位数
    pub exam_seat_count: u32,
}

/// 获取空教室信息
///
/// # Arguments
///
/// - `hdjw_token`: 教务系统的令牌，可以通过 [HdjwToken::acquire_by_cas_login] 获取
/// - `building_id`: 楼栋id，参考 `docs/hdjw/building.md` 的 `楼栋 id` 一栏
/// - `week`: 第几周
/// - `day`: 周几，星期一为 `1`，星期日为 `7`
/// - `time`: 节次信息。切片内的元素需要是大节次，参考 `docs/hdjw/time.md` 的 `大节次` 一栏。注意，不支持第 6 大节。
/// - `xn`: 学年
/// - `xq`: 学期
///
/// # Returns
///
/// 空教室列表
///
/// # Errors
///
/// 如果提供的 `hdjw_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [HdjwToken]
///
/// # Panics
///
/// `time` 必须位于区间 [1, 5] 内，否则会 panic
pub async fn get_empty_classroom(
    hdjw_token: &HdjwToken,
    building_id: &str,
    week: u8,
    day: u8,
    time: &[u8],
    xn: u16,
    xq: u8,
) -> Result<Vec<EmptyClassroom>, crate::Error<TokenExpired>> {
    let time_str = time
        .iter()
        .map(|&x| match x {
            1 => "0102",
            2 => "0304",
            3 => "0506",
            4 => "0708",
            5 => "091011",
            _ => panic!("不支持第 {} 大节", x),
        })
        .collect::<Vec<_>>()
        .join(",");
    let raw_data =
        raw::get_jsjy_query2(hdjw_token, xn, xq, week, day, &time_str, building_id).await?;
    parse::empty_classroom(raw_data, time)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hdjw::test::{
        TEST_HDJW_BUILDING_ID, TEST_HDJW_DAY_OF_WEEK, TEST_HDJW_TIME, TEST_HDJW_WEEK,
        get_hdjw_token,
    };
    use crate::test::{TEST_XN, TEST_XQ, TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_empty_classroom() -> TestResult<()> {
        let hdjw_token = get_hdjw_token().await?;
        let empty_classroom = get_empty_classroom(
            &hdjw_token,
            TEST_HDJW_BUILDING_ID,
            *TEST_HDJW_WEEK,
            *TEST_HDJW_DAY_OF_WEEK,
            &TEST_HDJW_TIME,
            *TEST_XN,
            *TEST_XQ,
        )
        .await?;
        println!("{:#?}", empty_classroom);
        Ok(())
    }
}
