use reqwest::StatusCode;
use reqwest::header::LOCATION;

use crate::{
    cg::{CgToken, error::CgError},
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
};

const BASE_URL: &str = "https://cg.hnu.edu.cn";

/// 进入课程上下文，跟随重定向以建立服务端 session 状态
pub(super) async fn enter_course_context(
    token: &CgToken,
    course_id: u32,
) -> Result<(), crate::Error<CgError>> {
    let res = client
        .get(format!("{}/courselist.jsp", BASE_URL))
        .query(&[("courseID", course_id.to_string())])
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;

    if res.status() == StatusCode::FOUND {
        let location = res
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if location.contains("simple.jsp") || location.contains("login") {
            return Err(crate::Error::Other(CgError::TokenExpired));
        }
        let redirect_url = if location.starts_with('/') {
            format!("{}{}", BASE_URL, location)
        } else {
            format!("{}/{}", BASE_URL, location)
        };
        client
            .get(&redirect_url)
            .headers(token.headers().clone())
            .send()
            .await
            .network_err()?;
    } else {
        return Err(format!("进入课程失败: HTTP {}", res.status())).unexpected_err();
    }
    Ok(())
}

/// 获取主页 HTML（单课程账号）
pub(super) async fn main_page(token: &CgToken) -> Result<String, crate::Error<CgError>> {
    client
        .get(format!("{}/main.jsp", BASE_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .text()
        .await
        .network_err()
}

/// 获取作业列表页
pub(super) async fn assignment_list_page(token: &CgToken) -> Result<String, crate::Error<CgError>> {
    client
        .get(format!("{}/assignment/mainActiveAssigns.jsp", BASE_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .text()
        .await
        .network_err()
}

/// 获取题目列表页 (`assignment/index.jsp?assignID=xx`)
pub(super) async fn problem_list_page(
    token: &CgToken,
    assign_id: u32,
) -> Result<String, crate::Error<CgError>> {
    client
        .get(format!("{}/assignment/index.jsp", BASE_URL))
        .query(&[("assignID", assign_id.to_string())])
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .text()
        .await
        .network_err()
}

/// 获取题目详情页 HTML，跟随 302 重定向
pub(super) async fn problem_page(
    token: &CgToken,
    assign_id: u32,
    pro_num: u32,
) -> Result<String, crate::Error<CgError>> {
    let res = client
        .get(format!("{}/assignment/programList.jsp", BASE_URL))
        .query(&[
            ("proNum", pro_num.to_string()),
            ("assignID", assign_id.to_string()),
        ])
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;

    let final_url = if res.status() == StatusCode::FOUND {
        let location = res
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if location.contains("simple.jsp") || location.contains("login") {
            return Err(crate::Error::Other(CgError::TokenExpired));
        }

        if location.starts_with('/') {
            format!("{}{}", BASE_URL, location)
        } else if location.starts_with("http") {
            location.to_string()
        } else {
            format!("{}/assignment/{}", BASE_URL, location)
        }
    } else {
        format!("{}/assignment/programList.jsp", BASE_URL)
    };

    client
        .get(&final_url)
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .text()
        .await
        .network_err()
}
