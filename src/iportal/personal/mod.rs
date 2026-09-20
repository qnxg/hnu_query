//! iportal 首页个人数据查询

mod fetch;
mod parse;

use crate::{
    error::MapUnexpectedErr,
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
    utils::obs::{fetch_time, parse_time},
};
use chrono::NaiveDateTime;
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

/// 当前账号首页个人数据的聚合结果。
///
/// 服务端未返回的项目为 `None`。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PersonalData {
    /// 图书馆借阅书籍数量
    pub lib_borrow: Option<u32>,
    /// 绑定的邮箱
    pub email: Option<String>,
    /// 未读邮件数量
    pub mail_unread: Option<u32>,
    /// 校园卡余额，单位为元
    pub balance: Option<f64>,
    /// 上次登录时间
    pub last_login_time: Option<NaiveDateTime>,
    /// 校园网已用流量，单位为 Byte
    pub net_used: Option<u64>,
}

/// 获取当前账号全部可用的个人数据。
///
/// 该函数会先获取数据项列表，再逐项获取详情；服务端未提供的项目保留为 `None`。
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过 [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login)
///
/// # Returns
///
/// 返回对应用户的个人数据
///
/// # Errors
///
/// `token` 失效时返回 [`IPortalTokenExpired`]
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_personal_data(
    token: &IPortalToken,
) -> Result<PersonalData, crate::Error<IPortalTokenExpired>> {
    let types_json = fetch_time!(fetch::personal_data_query_ids(token).await)?;
    let types = parse_time!(parse::personal_data_query_ids(&types_json))?;
    let raw_items = fetch_time!(
        async {
            let mut tasks = JoinSet::new();
            for (key, id) in types {
                let token = token.clone();
                tasks.spawn(async move {
                    let json = fetch::personal_data_single(&token, &id).await?;
                    Ok::<_, crate::Error<IPortalTokenExpired>>((key, json))
                });
            }

            let mut raw_items = Vec::new();
            while let Some(task) = tasks.join_next().await {
                raw_items.push(task.unexpected_err()??);
            }
            Ok::<_, crate::Error<IPortalTokenExpired>>(raw_items)
        }
        .await
    )?;
    parse_time!(parse::personal_data(raw_items))
}

#[cfg(test)]
mod tests {
    use crate::{
        iportal::{get_personal_data, test::get_iportal_token},
        test::TestResult,
    };

    #[tokio::test]
    #[ignore]
    async fn test_fetch_personal_data() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let summary = get_personal_data(&token).await?;
        println!("{summary:#?}");
        Ok(())
    }
}
