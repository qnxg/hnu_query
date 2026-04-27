use crate::{
    ca::login::CaToken,
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    utils::client,
};
use serde_json::Value;
use std::{convert::Infallible, time::Duration};

/// 本科生主修所有课程的中文成绩单
pub const UNDERGRADUATE_MAJOR_ALL_TEMPLATE_ID: &str = "02a70e11bc89b40dc2ef6ed14851ce25";

/// 获取可信电子凭证文件的文本原始数据
///
/// # Arguments
///
/// - `stu_id`: 学号
/// - `template_id`: 模板 id
///
/// # Returns
///
/// 可信电子凭证文件的 pdf 文本原始数据
pub async fn raw_certification_data(
    ca_token: &CaToken,
    template_id: &str,
) -> Result<String, crate::Error<Infallible>> {
    let template_url = format!(
        "https://ca.hnu.edu.cn/student/student/caTemplate/preview_file?templateId={}&isbzf=0&kcxz=&xfjd=&xzkc=",
        template_id
    );
    let json_str = client
        .get(&template_url)
        .timeout(Duration::from_secs(60))
        .headers(ca_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .text()
        .await
        .unexpected_err()?;
    let json: Value = serde_json::from_str(&json_str).parse_err(&json_str)?;
    if json.get("code").and_then(|v| v.as_u64()) != Some(200) {
        return Err(parse_err(&json_str));
    }
    let Some(file_name) = json.get("message").and_then(|v| v.as_str()) else {
        return Err(parse_err(&json_str));
    };
    let file_url = format!(
        "https://ca.hnu.edu.cn/student/sys/common/view/{}",
        file_name
    );
    // 下载文件
    let res = client
        .get(&file_url)
        .timeout(Duration::from_secs(60))
        .headers(ca_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?;
    let bytes = res.bytes().await.unexpected_err()?;
    let pdf = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    Ok(pdf)
}
