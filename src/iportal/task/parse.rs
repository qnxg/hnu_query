use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::{
    error::parse_err,
    iportal::{
        error::IPortalTokenExpired,
        task::{ApplyItem, ApplyList, ApplyStatus},
        util::iportal_jsondata_precheck,
    },
};

/// 一条原始流程申请记录
///
/// 包含申请所对应的应用、发起人、部门、处理进度、时间以及详情链接等信息
#[derive(Debug, Deserialize)]
#[expect(unused)]
struct RawApplyItem {
    id: i64,
    apps_id: i32,
    app_name: String,
    name: String,
    creator: i32,
    number: String,
    inst_created: String,
    inst_finished: Option<String>,
    inst_status: i8,
    percent: u8,
    created: String,
    updated: String,
    department_name: String,
    department_id: i32,
    department_sn: String,
    form_url_view: String,
    form_mobile_url_view: String,
    process_pic_url: String,
    process_log_url: String,
    custom_status: String,
    del_uid: i32,
    del_time: Option<String>,
    third_id: i32,
    third_app_id: String,
    third_app_name: String,
    third_inst_id: String,
    third_inst_name: String,
    third_name: String,
}

pub fn parse_apply_list(json_str: &str) -> Result<ApplyList, crate::Error<IPortalTokenExpired>> {
    let data = iportal_jsondata_precheck(json_str)?;
    let total = data
        .get("total")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| parse_err("无法解析 total 字段", json_str))?;
    let list = data
        .get("list")
        .and_then(|v| v.as_array())
        .ok_or_else(|| parse_err("无法解析 list 字段", json_str))?
        .iter()
        .map(|item| {
            let raw_item: RawApplyItem = serde_json::from_value(item.clone())
                .map_err(|e| parse_err(&format!("无法解析申请记录: {}", e), json_str))?;
            Ok(ApplyItem {
                app_name: raw_item.app_name,
                creator_name: raw_item.name,
                creator_department: raw_item.department_name,
                created: NaiveDateTime::parse_from_str(&raw_item.created, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| parse_err(&format!("无法解析创建时间: {}", e), json_str))?,
                finished: raw_item
                    .inst_finished
                    .map(|value| {
                        NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S")
                            .map_err(|e| parse_err(&format!("无法解析完成时间: {}", e), json_str))
                    })
                    .transpose()?,
                id: raw_item.id,
                status: match raw_item.inst_status {
                    0 => ApplyStatus::Pending,
                    1 => ApplyStatus::Processing,
                    2 => ApplyStatus::Completed,
                    other => ApplyStatus::Other(other),
                },
            })
        })
        .collect::<Result<Vec<ApplyItem>, _>>()?;
    Ok(ApplyList { total, list })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_apply_list() -> TestResult<()> {
        let apply_list = parse_apply_list(include_str!("test_data/apply_list.json"))?;

        assert_eq!(apply_list.total, 1);
        assert_eq!(apply_list.list.len(), 1);

        let item = &apply_list.list[0];
        assert_eq!(item.id, 100001);
        assert_eq!(item.app_name, "测试事项");
        assert_eq!(item.creator_name, "测试用户");
        assert_eq!(item.creator_department, "测试学院");
        assert_eq!(
            item.created,
            NaiveDateTime::parse_from_str("2024-01-01 10:00:01", "%Y-%m-%d %H:%M:%S")?
        );
        assert_eq!(
            item.finished,
            Some(NaiveDateTime::parse_from_str(
                "2024-01-01 10:30:00",
                "%Y-%m-%d %H:%M:%S",
            )?)
        );
        assert!(matches!(item.status, ApplyStatus::Completed));

        Ok(())
    }
}
