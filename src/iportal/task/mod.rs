//! 流程申请记录查询

use crate::{
    iportal::error::IPortalTokenExpired,
    utils::obs::{fetch_time, parse_time},
};
use chrono::NaiveDateTime;
use hnu_query_macros::traced;
use serde::{Deserialize, Serialize};

mod fetch;
mod parse;

/// 分页的申请记录列表
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApplyList {
    /// 申请记录总数
    pub total: i64,
    /// 当前页的申请记录
    pub list: Vec<ApplyItem>,
}

/// 申请记录
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApplyItem {
    /// 申请记录ID
    pub id: i64,
    /// 对应申请名称
    pub app_name: String,
    /// 发起人姓名
    pub creator_name: String,
    /// 发起人所属部门, 例如: `计算机学院（软件学院、国家保密学院）`
    pub creator_department: String,
    /// 发起时间, 格式`yyyy-MM-dd HH:mm:ss`
    pub created: NaiveDateTime,
    /// 完成时间, 格式`yyyy-MM-dd HH:mm:ss`, 可能为 `null`
    pub finished: Option<NaiveDateTime>,
    /// 当前申请状态,
    pub status: ApplyStatus,
}

#[repr(i8)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ApplyStatus {
    /// 已提交，但尚未开始处理
    Pending = 0,
    /// 处理中
    Processing = 1,
    /// 已完成
    Completed = 2,
    /// 未知
    Other(i8),
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
/// 当令牌失效、网络请求失败或响应无法解析时返回错误
#[traced(subsystem = "iportal", skip(token))]
pub async fn get_apply_list(
    token: &crate::iportal::login::IPortalToken,
    page: i32,
    page_size: i32,
) -> Result<ApplyList, crate::Error<IPortalTokenExpired>> {
    let json_str = fetch_time!(fetch::fetch_apply_list(token, page, page_size).await)?;
    let task_list = parse_time!(parse::parse_apply_list(&json_str))?;
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
