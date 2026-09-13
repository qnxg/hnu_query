use crate::{
    error::parse_err,
    iportal::{
        personal::{PersonalDataItem, PersonalDataTypeEnum},
        util::iportal_jsondata_precheck,
    },
};

pub fn parse_personal_data_query_ids(
    json_str: &str,
) -> Result<Vec<PersonalDataTypeEnum>, crate::Error<crate::cas::error::TokenExpired>> {
    let json_value = iportal_jsondata_precheck(json_str);
    let arr = json_value?
        .get("data")
        .ok_or(parse_err("JSON解析错误", json_str))?
        .as_array()
        .ok_or(parse_err("JSON解析错误", json_str))?
        .clone();
    let mut items = Vec::new();
    for item in arr {
        let data_item: PersonalDataTypeEnum = serde_json::from_value(item.clone())
            .map_err(|e| parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
        items.push(data_item);
    }
    Ok(items)
}

pub fn parse_personal_data(
    json_str: &str,
) -> Result<PersonalDataItem, crate::Error<crate::cas::error::TokenExpired>> {
    let json_value: Result<serde_json::Value, crate::Error<crate::cas::error::TokenExpired>> =
        iportal_jsondata_precheck(json_str);
    let data_item = json_value?
        .get("data")
        .ok_or(parse_err("JSON解析错误", json_str))?
        .clone();
    let item: PersonalDataItem = serde_json::from_value(data_item)
        .map_err(|e| parse_err(format!("JSON解析错误: {}", e).as_str(), json_str))?;
    Ok(item)
}
