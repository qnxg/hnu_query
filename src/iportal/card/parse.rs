use crate::{
    error::{MapParseErr, parse_err},
    iportal::{
        card::{CardInfo, CardTransactionDetail, TransactionRecord, TransactionType},
        error::IPortalTokenExpired,
        util::iportal_jsondata_precheck,
    },
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawTransactionRecord {
    // 原始交易名称
    tranname: String,
    // 商户名称
    mercname: String,
    // 商户账号
    mercacc: String,
    // 交易时间, 格式为 `yyyyMMddHHmmss`
    occtime: String,
    // 带收支方向的交易金额, 单位为分
    sign_tranamt: String,
    // 卡余额快照, 注意单位为分
    cardbal: String,
    // 交易类型代码
    trancode: String,
}

/// 解析校园卡账户信息响应
///
/// # Arguments
///
/// - `json_str`: [`super::fetch::balance`] 返回的数据
pub fn card_info(json_str: &str) -> Result<CardInfo, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str)?;
    let info = serde_json::from_value(json_value).parse_err(json_str)?;
    Ok(info)
}

/// 解析校园卡交易明细响应
///
/// # Arguments
///
/// - `json_str`: [`super::fetch::transaction_records`] 返回的数据
pub fn card_transaction_records(
    json_str: &str,
) -> Result<CardTransactionDetail, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str)?;
    let next_page = json_value
        .get("nextpage")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| parse_err("无法解析 nextpage 字段", json_str))?
        .parse::<u32>()
        .parse_err(json_str)?;
    let row_count = json_value
        .get("rowcount")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| parse_err("无法解析 rowcount 字段", json_str))?
        .parse::<u32>()
        .parse_err(json_str)?;
    let records = json_value
        .get("total")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| parse_err("无法解析 total 字段为数组", json_str))?
        .iter()
        .map(|item| {
            let code = item
                .get("trancode")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| parse_err("无法解析 trancode 字段", json_str))?;
            // 跳过一些卡片管理操作：
            // 03: 持卡人信息变更, 41: 持卡人换卡, 42: 挂失,
            // 43: 解挂, 44: 冻结, 5K: 卡片校正（身份和有效期）
            if matches!(code, "03" | "41" | "42" | "43" | "44" | "5K") {
                return Ok(None);
            }

            let raw_item: RawTransactionRecord =
                serde_json::from_value(item.clone()).parse_err(json_str)?;
            let transaction_type = match raw_item.trancode.as_str() {
                "15" => TransactionType::Consumption,
                "16" => TransactionType::Recharge,
                "17" => TransactionType::Benefit,
                "22" => TransactionType::MoneyTransfer,
                "6" => TransactionType::EAccountOpening,
                "1" => TransactionType::HolderAccountOpening,
                "99" => TransactionType::QRCodeConsumption,
                "27" => TransactionType::Withholding,
                _ => {
                    return Err(parse_err(
                        &format!("未知交易类型代码: {}", raw_item.trancode),
                        json_str,
                    ));
                }
            };
            Ok(Some(TransactionRecord {
                transaction_name: raw_item.tranname,
                merchant_name: raw_item.mercname,
                merchant_account: raw_item.mercacc.parse::<u64>().parse_err(json_str)?,
                pay_time: chrono::NaiveDateTime::parse_from_str(&raw_item.occtime, "%Y%m%d%H%M%S")
                    .parse_err(json_str)?,
                transaction_amount: raw_item.sign_tranamt.parse::<i64>().parse_err(json_str)?,
                card_balance: raw_item.cardbal.parse::<i64>().parse_err(json_str)?,
                transaction_type,
            }))
        })
        .collect::<Result<Vec<_>, crate::Error<IPortalTokenExpired>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(CardTransactionDetail {
        has_next_page: next_page != 0,
        total_count: row_count,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_card_balance() -> TestResult<()> {
        let balance = card_info(include_str!("test_data/balance.json"))?;

        assert_eq!(balance.account, "114514");
        assert_eq!(balance.balance, 1919.81);

        Ok(())
    }

    #[test]
    fn test_parse_card_transaction_records() -> TestResult<()> {
        let result = card_transaction_records(include_str!("test_data/transactions.json"))?;
        assert!(result.has_next_page);
        assert_eq!(result.total_count, 11);
        assert_eq!(result.records.len(), 10);
        assert_eq!(result.records[0].merchant_name, "天马二食堂二楼");
        assert_eq!(result.records[0].transaction_name, "持卡人消费");
        assert_eq!(result.records[0].card_balance, 8181);
        assert_eq!(
            result.records[0].transaction_type,
            TransactionType::Consumption
        );

        Ok(())
    }

    #[test]
    fn test_card_transaction_records_skip_card_management() -> TestResult<()> {
        let result =
            card_transaction_records(include_str!("test_data/transactions_card_management.json"))?;

        assert_eq!(result.total_count, 7);
        assert_eq!(result.records.len(), 1);
        assert_eq!(
            result.records[0].transaction_type,
            TransactionType::Consumption
        );
        Ok(())
    }

    #[test]
    fn test_card_transaction_records_unknown_code() {
        let result = card_transaction_records(
            r#"{
                "e": 0,
                "d": {
                    "nextpage": "0",
                    "rowcount": "1",
                    "total": [{
                        "trancode": "45",
                        "tranname": "未知交易",
                        "mercacc": "0",
                        "mercname": "",
                        "sign_tranamt": "0",
                        "cardbal": "0",
                        "occtime": "20261101000000"
                    }]
                }
            }"#,
        );

        assert!(matches!(result, Err(crate::Error::Parse(_))));
    }

    #[test]
    fn test_parse_card_error_response() -> TestResult<()> {
        let result = card_info(
            r#"{
                "e": 1,
                "m": "参数错误",
                "d": null
            }"#,
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_card_transaction_records_without_next_page() -> TestResult<()> {
        let result =
            card_transaction_records(r#"{"e":0,"d":{"nextpage":"0","rowcount":"0","total":[]}}"#)?;

        assert!(!result.has_next_page);
        assert_eq!(result.total_count, 0);
        assert!(result.records.is_empty());
        Ok(())
    }

    #[test]
    fn test_card_info_malformed_json() -> TestResult<()> {
        assert!(card_info(include_str!("../test_data/malformed.json")).is_err());
        Ok(())
    }

    #[test]
    fn test_card_transaction_records_invalid_time() -> TestResult<()> {
        let result =
            card_transaction_records(include_str!("test_data/transactions_invalid_time.json"));

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_card_transaction_records_invalid_shape() -> TestResult<()> {
        let result =
            card_transaction_records(include_str!("test_data/transactions_invalid_shape.json"));

        assert!(result.is_err());
        Ok(())
    }
}
