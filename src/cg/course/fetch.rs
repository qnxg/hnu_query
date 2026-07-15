use crate::{
    cg::{CgToken, error::CgError},
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
};
use reqwest::{StatusCode, header::LOCATION};

const BASE_URL: &str = "https://cg.hnu.edu.cn";
const COURSELIST_URL: &str = "/courselist.jsp";
const MAIN_URL: &str = "/main.jsp";
const ASSIGNMENT_LIST_URL: &str = "/assignment/mainActiveAssigns.jsp";
const PROBLEM_LIST_URL: &str = "/assignment/index.jsp";
const PROBLEM_PAGE_URL: &str = "/assignment/programList.jsp";

/// 进入课程上下文，跟随重定向以建立服务端 session 状态
pub(super) async fn enter_course_context(
    token: &CgToken,
    course_id: u32,
) -> Result<(), crate::Error<CgError>> {
    let res = client
        .get(format!("{}{}", BASE_URL, COURSELIST_URL))
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

/// 获取主页 HTML（单课程账号），`html` 为 [`super::parse::parse_courses_from_main`] 的输入数据
pub(super) async fn main_page(token: &CgToken) -> Result<String, crate::Error<CgError>> {
    client
        .get(format!("{}{}", BASE_URL, MAIN_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .text()
        .await
        .unexpected_err()
}

/// 获取作业列表页 HTML，`html` 为 [`super::parse::parse_assignments`] 的输入数据
pub(super) async fn assignment_list_page(token: &CgToken) -> Result<String, crate::Error<CgError>> {
    client
        .get(format!("{}{}", BASE_URL, ASSIGNMENT_LIST_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .text()
        .await
        .unexpected_err()
}

/// 获取题目列表页 HTML (`assignment/index.jsp?assignID=xx`)，
/// `html` 为 [`super::parse::parse_problems`] 的输入数据
pub(super) async fn problem_list_page(
    token: &CgToken,
    assign_id: u32,
) -> Result<String, crate::Error<CgError>> {
    client
        .get(format!("{}{}", BASE_URL, PROBLEM_LIST_URL))
        .query(&[("assignID", assign_id.to_string())])
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .text()
        .await
        .unexpected_err()
}

/// 获取题目详情页 HTML，跟随 302 重定向
pub(super) async fn problem_page(
    token: &CgToken,
    assign_id: u32,
    pro_num: u32,
) -> Result<String, crate::Error<CgError>> {
    let res = client
        .get(format!("{}{}", BASE_URL, PROBLEM_PAGE_URL))
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
        format!("{}{}", BASE_URL, PROBLEM_PAGE_URL)
    };

    client
        .get(&final_url)
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?
        .text()
        .await
        .unexpected_err()
}
