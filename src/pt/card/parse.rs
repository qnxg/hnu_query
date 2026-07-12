use super::{CardHistory, CardHistoryItem, CardInfo};
use crate::error::{MapParseErr, parse_err};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

#[derive(Deserialize, Debug)]
struct RawCardInfo {
    account: u32,
    balance: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawCardHistory {
    amt: f64,
    count: f64,
    webTrjnDTO: Option<Vec<RawCardHistoryItem>>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct RawCardHistoryItem {
    fTranAmt: String,
    jndatetime: String,
    effectdate: String,
    jourName: String,
    usedcardnum: u32,
    nowAmt: String,
    sysname1: Option<String>,
    tranname: String,
}

/// `json_str` 为 [super::fetch::csrf_token] 的返回数据
pub fn csrf_token(json_str: &str) -> Result<String, crate::Error<Infallible>> {
    serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| parse_err(json_str))
}

/// `json_str` 为 [super::fetch::card_info] 的返回数据
pub fn card_info(json_str: &str) -> Result<CardInfo, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .map(|v| serde_json::from_value::<RawCardInfo>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;

    let raw_balance = raw_data
        .balance
        .parse::<f64>()
        .parse_err(&raw_data.balance)?;

    Ok(CardInfo {
        id: raw_data.account,
        balance: raw_balance / 100.0,
    })
}

/// `json_str` 为 [super::fetch::card_history] 的返回数据
pub fn card_history(json_str: &str) -> Result<CardHistory, crate::Error<Infallible>> {
    let raw_data = serde_json::from_str::<Value>(json_str)
        .parse_err(json_str)?
        .get("data")
        .map(|v| serde_json::from_value::<RawCardHistory>(v.clone()).parse_err(json_str))
        .transpose()?
        .ok_or_else(|| parse_err(json_str))?;
    let raw_items = raw_data.webTrjnDTO.unwrap_or_default();
    let mut items = Vec::with_capacity(raw_items.len());
    for item in raw_items {
        let date_time = NaiveDateTime::parse_from_str(&item.effectdate, "%Y/%m/%d %H:%M:%S")
            .parse_err_with_reason(&item.effectdate, "date_time")?;
        let journal_time = NaiveDateTime::parse_from_str(&item.jndatetime, "%Y/%m/%d %H:%M:%S")
            .parse_err_with_reason(&item.jndatetime, "journal_time")?;
        let now_balance = item
            .nowAmt
            .trim()
            // 可能会有 1,359.30 这种情况
            .replace([',', ' '], "")
            .parse::<f64>()
            .parse_err_with_reason(&item.nowAmt, "now_balance")?;
        let amount = item
            .fTranAmt
            .parse::<f64>()
            .parse_err_with_reason(&item.fTranAmt, "amount")?;
        items.push(CardHistoryItem {
            date_time,
            journal_time,
            status: item.jourName,
            id: item.usedcardnum,
            now_balance,
            amount,
            location: item.sysname1.map(|s| s.trim().to_string()),
            name: item.tranname,
        });
    }

    Ok(CardHistory {
        total: raw_data.amt / 100.0,
        count: raw_data.count as u32,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;
    use chrono::NaiveDate;

    #[test]
    fn test_card_info() -> TestResult<()> {
        let info = card_info(include_str!("test_data/getCardUserInfo.json"))?;
        assert_eq!(info.id, 123456);
        assert_eq!(info.balance, 75.28);
        Ok(())
    }

    #[test]
    fn test_card_history_consumption() -> TestResult<()> {
        let history = card_history(include_str!(
            "test_data/getAccHisConsubDzzfLog_Consumption.json"
        ))?;

        assert_eq!(history.count, 7);
        assert_eq!(history.total, -51.0);
        assert_eq!(history.items.len(), 7);
        // 仅测试第一项通过即可
        let first_item = &history.items[0];
        assert_eq!(
            first_item.date_time,
            NaiveDate::from_ymd_opt(2026, 5, 30)
                .expect("this should not panic")
                .and_hms_opt(17, 38, 44)
                .expect("this should not panic")
        );
        assert_eq!(
            first_item.journal_time,
            NaiveDate::from_ymd_opt(2026, 5, 30)
                .expect("this should not panic")
                .and_hms_opt(17, 38, 42)
                .expect("this should not panic")
        );
        assert_eq!(first_item.status, "正常".to_string());
        assert_eq!(first_item.id, 56);
        assert_eq!(first_item.now_balance, 71.43);
        assert_eq!(first_item.location, Some("天马二食堂二楼".to_string()));
        assert_eq!(first_item.name, "持卡人消费");

        Ok(())
    }

    #[test]
    fn test_card_history_recharge() -> TestResult<()> {
        let history = card_history(include_str!(
            "test_data/getAccHisConsubDzzfLog_Recharge.json"
        ))?;

        assert_eq!(history.count, 3);
        assert_eq!(history.total, 50.00);
        assert_eq!(history.items.len(), 3);
        // 仅测试第一项通过即可
        let first_item = &history.items[0];
        assert_eq!(
            first_item.date_time,
            NaiveDate::from_ymd_opt(2026, 5, 25)
                .expect("this should not panic")
                .and_hms_opt(11, 57, 41)
                .expect("this should not panic")
        );
        assert_eq!(
            first_item.journal_time,
            NaiveDate::from_ymd_opt(2026, 5, 25)
                .expect("this should not panic")
                .and_hms_opt(11, 57, 41)
                .expect("this should not panic")
        );
        assert_eq!(first_item.status, "正常".to_string());
        assert_eq!(first_item.id, 50);
        assert_eq!(first_item.now_balance, 90.78);
        assert_eq!(first_item.location, None);
        assert_eq!(first_item.name, "银行转账");

        Ok(())
    }
}
