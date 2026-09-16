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
    // 接口原始商户名称
    mercname: String,
    // 接口原始商户账号
    mercacc: String,
    // 接口原始交易时间, 格式为 `yyyyMMddHHmmss`
    occtime: String,
    // 接口原始带收支方向的交易金额, 单位为分
    sign_tranamt: String,
    // 接口原始交易类型代码
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
            let raw_item: RawTransactionRecord =
                serde_json::from_value(item.clone()).parse_err(json_str)?;
            let code = raw_item.trancode.parse::<u32>().parse_err(json_str)?;
            Ok(TransactionRecord {
                merchant_name: raw_item.mercname,
                merchant_account: raw_item.mercacc.parse::<u64>().parse_err(json_str)?,
                pay_time: chrono::NaiveDateTime::parse_from_str(&raw_item.occtime, "%Y%m%d%H%M%S")
                    .parse_err(json_str)?,
                transaction_amount: raw_item.sign_tranamt.parse::<i64>().parse_err(json_str)?,
                transaction_type: match code {
                    15 => TransactionType::Consumption,
                    16 => TransactionType::Recharge,
                    17 => TransactionType::Benefit,
                    22 => TransactionType::MoneyTransfer,
                    6 => TransactionType::EAccountOpening,
                    1 => TransactionType::HolderAccountOpening,
                    _ => TransactionType::Other(code),
                },
            })
        })
        .collect::<Result<Vec<TransactionRecord>, crate::Error<IPortalTokenExpired>>>()?;
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
        let detail = card_transaction_records(include_str!("test_data/transactions.json"))?;

        assert!(detail.has_next_page);
        assert_eq!(detail.total_count, 11);
        assert_eq!(detail.records.len(), 10);

        let first = &detail.records[0];
        assert_eq!(first.merchant_name, "天马二食堂二楼");
        assert_eq!(first.merchant_account, 1000006);
        assert_eq!(
            first.pay_time,
            chrono::NaiveDateTime::parse_from_str("2026-09-13 11:47:30", "%Y-%m-%d %H:%M:%S")?
        );
        assert_eq!(first.transaction_amount, -200);
        assert!(matches!(
            first.transaction_type,
            crate::iportal::card::TransactionType::Consumption
        ));

        let transfer = &detail.records[4];
        assert_eq!(transfer.transaction_amount, -300);
        assert!(matches!(
            transfer.transaction_type,
            crate::iportal::card::TransactionType::MoneyTransfer
        ));

        let unknown = &detail.records[5];
        assert!(matches!(
            unknown.transaction_type,
            crate::iportal::card::TransactionType::Other(27)
        ));

        Ok(())
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
