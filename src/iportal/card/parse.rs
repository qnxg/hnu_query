use serde::Deserialize;

use crate::{
    error::parse_err,
    iportal::{error::IPortalTokenExpired, util::iportal_jsondata_precheck},
};

#[derive(Debug, Clone, Deserialize)]
#[expect(unused)]
struct RawTransactionRecord {
    /// 接口原始商户名称
    mercname: String,
    /// 接口原始商户账号
    mercacc: String,
    /// 接口原始交易时间，格式为 `yyyyMMddHHmmss`
    occtime: String,
    /// 接口原始带收支方向的交易金额，单位为分
    sign_tranamt: String,
    /// 接口原始交易金额，单位为分
    tranamt: String,
    /// 接口原始交易类型代码
    trancode: String,
    /// 接口原始交易类型名称
    tranname: String,
}
pub fn parse_card_balance(
    json_str: &str,
) -> Result<crate::iportal::card::CardInfo, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str)?;
    let balance = serde_json::from_value(json_value)
        .map_err(|e| parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
    Ok(balance)
}

pub fn parse_card_transaction_records(
    json_str: &str,
) -> Result<crate::iportal::card::CardTransactionDetail, crate::Error<IPortalTokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str)?;
    let next_page = json_value
        .get("nextpage")
        .and_then(|v| v.as_str())
        .ok_or_else(|| parse_err("无法解析 nextpage 字段", json_str))?
        .to_string()
        .parse::<u32>()
        .map_err(|e| {
            parse_err(
                format!("无法解析 nextpage 字段为数字: {}", e).as_str(),
                json_str,
            )
        })?;
    let page_size = json_value
        .get("pagesize")
        .and_then(|v| v.as_str())
        .ok_or_else(|| parse_err("无法解析 pagesize 字段", json_str))?
        .to_string()
        .parse::<u32>()
        .map_err(|e| {
            parse_err(
                format!("无法解析 pagesize 字段为数字: {}", e).as_str(),
                json_str,
            )
        })?;
    let row_count = json_value
        .get("rowcount")
        .and_then(|v| v.as_str())
        .ok_or_else(|| parse_err("无法解析 rowcount 字段", json_str))?
        .to_string()
        .parse::<u32>()
        .map_err(|e| {
            parse_err(
                format!("无法解析 rowcount 字段为数字: {}", e).as_str(),
                json_str,
            )
        })?;
    let data =
        json_value
            .get("total")
            .ok_or_else(|| parse_err("无法解析 data 字段", json_str))?
            .as_array()
            .ok_or_else(|| parse_err("无法解析 data 字段为数组", json_str))?
            .iter()
            .map(|item| {
                let raw_item: RawTransactionRecord = serde_json::from_value(item.clone())
                    .map_err(|e| parse_err(&format!("无法解析交易记录: {}", e), json_str))?;
                Ok(crate::iportal::card::TransactionRecord {
                    merchant_name: raw_item.mercname,
                    merchant_account: raw_item.mercacc.parse::<u64>().map_err(|e| {
                        parse_err(format!("无法解析商户账号为数字: {}", e).as_str(), json_str)
                    })?,
                    pay_time: chrono::NaiveDateTime::parse_from_str(
                        &raw_item.occtime,
                        "%Y%m%d%H%M%S",
                    )
                    .map_err(|e| {
                        parse_err(
                            &format!("无法解析交易时间 {}: {}", raw_item.occtime, e),
                            json_str,
                        )
                    })?,
                    transaction_amount: raw_item.sign_tranamt.parse::<i64>().map_err(|e| {
                        parse_err(format!("无法解析交易金额为数字: {}", e).as_str(), json_str)
                    })?,
                    transaction_type: match raw_item.trancode.parse::<u32>() {
                        Ok(code) => match code {
                            15 => crate::iportal::card::TransactionType::Consumption,
                            16 => crate::iportal::card::TransactionType::Recharge,
                            17 => crate::iportal::card::TransactionType::Benefit,
                            22 => crate::iportal::card::TransactionType::MoneyTransfer,
                            6 => crate::iportal::card::TransactionType::EAccountOpening,
                            1 => crate::iportal::card::TransactionType::HolderAccountOpening,
                            _ => crate::iportal::card::TransactionType::Other(code),
                        },
                        Err(e) => {
                            return Err(parse_err(
                                format!("无法解析交易类型代码为数字: {}", e).as_str(),
                                json_str,
                            ));
                        }
                    },
                })
            })
            .collect::<Result<
                Vec<crate::iportal::card::TransactionRecord>,
                crate::Error<IPortalTokenExpired>,
            >>()?;
    Ok(crate::iportal::card::CardTransactionDetail {
        next_page,
        page_size,
        row_count,
        records: data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_card_balance() -> TestResult<()> {
        let balance = parse_card_balance(include_str!("test_data/balance.json"))?;

        assert_eq!(balance.account, "114514");
        assert_eq!(balance.balance, 1919.81);

        Ok(())
    }

    #[test]
    fn test_parse_card_transaction_records() -> TestResult<()> {
        let detail = parse_card_transaction_records(include_str!("test_data/transactions.json"))?;

        assert_eq!(detail.next_page, 2);
        assert_eq!(detail.page_size, 10);
        assert_eq!(detail.row_count, 11);
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
    fn test_parse_card_error_response() {
        let result = parse_card_balance(
            r#"{
                "e": 1,
                "m": "参数错误",
                "d": null
            }"#,
        );

        assert!(result.is_err());
    }
}
