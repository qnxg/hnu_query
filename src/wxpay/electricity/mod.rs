mod raw;
mod utils;

use crate::{
    error::MapUnexpectedErr,
    wxpay::electricity::{raw::raw_electricity_data, utils::parse_dormitory},
    xgxt::personal_info::Dormitory,
};
use std::convert::Infallible;

/// 获取宿舍电量
///
/// # Parameters
///
/// - `dormitory`: 宿舍信息，可以通过 [`crate::xgxt::get_person_info`] 获取
///
/// # Returns
///
/// 返回电量信息，含单位，如 `"146.88度"`。
///
/// 获取到的电量信息的单位不固定，有时候为 `度`，有时候为 `元`，还可能存在其他未知情况。
///
/// # Panics
///
/// 请确保 `dormitory` 的解析是成功的，否则会 panic。你可以使用 [`crate::xgxt::personal_info::Dormitory::successfully_parsed`] 判断是否解析成功。
pub async fn get_electricity(dormitory: Dormitory) -> Result<String, crate::Error<Infallible>> {
    assert!(
        dormitory.successfully_parsed(),
        "参数 dormitory 必须成功解析"
    );
    let (park, build, room) = parse_dormitory(dormitory)?;
    match build.as_str() {
        // 望麓桥学生公寓的2栋和3栋无法区分南边还是北面
        // 考虑到同一个宿舍号不可能既是南又是北，所以我们两个都试试，取成功的
        "#2栋" | "#3栋" => {
            let res_north = raw_electricity_data(
                park,
                match build.as_str() {
                    "#2栋" => "52",
                    "#3栋" => "54",
                    _ => unreachable!(),
                },
                room.as_str(),
            )
            .await;
            let res_south = raw_electricity_data(
                park,
                match build.as_str() {
                    "#2栋" => "53",
                    "#3栋" => "55",
                    _ => unreachable!(),
                },
                room.as_str(),
            )
            .await;
            match (res_north, res_south) {
                (Ok(n), Err(_)) => {
                    // 这样做是为了给编译器类型推断提示
                    let n: String = n;
                    Ok(n)
                }
                (Err(_), Ok(s)) => {
                    let s: String = s;
                    Ok(s)
                }
                _ => Err("获取电量信息失败，无法区分宿舍南北").unexpected_err(),
            }
        }
        _ => raw_electricity_data(park, build.as_str(), room.as_str()).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_get_electricity() {
        let park = env!("TEST_DORMITORY_PARK");
        let build = env!("TEST_DORMITORY_BUILD");
        let room = env!("TEST_DORMITORY_ROOM");
        let dormitory = Dormitory::from_parsed_value(park, build, room);
        let electricity = get_electricity(dormitory).await.unwrap();
        println!("{:#?}", electricity);
    }
}
