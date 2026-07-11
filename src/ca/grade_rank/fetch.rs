use crate::{
    ca::login::CaToken,
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
};
use bytes::Bytes;
use std::{convert::Infallible, time::Duration};

pub async fn preview_file(
    ca_token: &CaToken,
    template_id: &str,
) -> Result<String, crate::Error<Infallible>> {
    let template_url = format!(
        "https://ca.hnu.edu.cn/student/student/caTemplate/preview_file?templateId={}&isbzf=0&kcxz=&xfjd=&xzkc=",
        template_id
    );
    client
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
        .unexpected_err()
}

/// `file_name` 来自 [super::parse::preview_file_name] 的返回值
pub async fn file(ca_token: &CaToken, file_name: &str) -> Result<Bytes, crate::Error<Infallible>> {
    let file_url = format!(
        "https://ca.hnu.edu.cn/student/sys/common/view/{}",
        file_name
    );
    // 下载文件
    client
        .get(&file_url)
        .timeout(Duration::from_secs(60))
        .headers(ca_token.headers().clone())
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .bytes()
        .await
        .unexpected_err()
}
