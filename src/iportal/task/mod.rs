//! 流程申请记录查询

mod fetch;
mod parse;

use crate::{
    iportal::error::IPortalTokenExpired,
    utils::obs::{fetch_time, parse_time},
};
use chrono::NaiveDateTime;
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};

/// 分页的申请记录列表
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApplyList {
    /// 总申请记录总数, **注意: 不是当前页的记录数**
    pub total: u32,
    /// 当前页的申请记录
    pub list: Vec<ApplyItem>,
}

/// 申请记录
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApplyItem {
    /// 申请记录ID, 用于区分不同申请记录
    pub id: u32,
    /// 对应申请操作的名称, 如 `校园卡及个人门户照片自助更换`
    pub app_name: String,
    /// 发起人姓名
    pub creator_name: String,
    /// 发起人所属部门, 例如: `计算机学院（软件学院、国家保密学院）`
    pub creator_department: String,
    /// 发起时间
    pub created: NaiveDateTime,
    /// 完成时间, 在没有完成时可能为 `null`
    pub finished: Option<NaiveDateTime>,
    /// 当前申请状态,
    pub status: ApplyStatus,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// 流程申请的处理状态
pub enum ApplyStatus {
    /// 已提交，但尚未开始处理
    Pending,
    /// 处理中
    Processing,
    /// 已完成
    Completed,
}

/// 分页获取当前账号发起的全部流程申请
///
/// # Arguments
///
/// - `token`: iportal令牌，可以通过
///   [`IPortalToken::acquire_by_cas_login`](crate::iportal::login::IPortalToken::acquire_by_cas_login)
///   获取
/// - `page`: 页码
/// - `page_size`: 每页记录数
///
/// # Returns
///
/// 返回包含记录总数和当前页记录的 [`ApplyList`]
///
/// # Errors
///
/// `token` 失效时返回 [`IPortalTokenExpired`]
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_apply_list(
    token: &crate::iportal::login::IPortalToken,
    page: u32,
    page_size: u32,
) -> Result<ApplyList, crate::Error<IPortalTokenExpired>> {
    let json_str = fetch_time!(fetch::apply_list(token, page, page_size).await)?;
    let task_list = parse_time!(parse::apply_list(&json_str))?;
    Ok(task_list)
}

#[cfg(test)]
mod tests {
    use crate::{
        iportal::{login::get_iportal_token, task::get_apply_list},
        test::TestResult,
    };

    #[tokio::test]
    #[ignore]
    async fn test_fetch_apply_list() -> TestResult<()> {
        let token = get_iportal_token().await?;
        let task_list = get_apply_list(&token, 1, 10).await?;
        println!("{task_list:#?}");
        Ok(())
    }
}
