use crate::{
    cg::{error::TokenExpired, login::CgToken},
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
};
use reqwest::{Response, StatusCode, header::LOCATION};

const BASE_URL: &str = "https://cg.hnu.edu.cn";
const COURSELIST_URL: &str = "/courselist.jsp";
const MAIN_URL: &str = "/main.jsp";
const ASSIGNMENT_LIST_URL: &str = "/assignment/mainActiveAssigns.jsp";
const PROBLEM_LIST_URL: &str = "/assignment/index.jsp";

/// 检查响应是否指示登录已过期（跳转到了登录页）
fn check_token_expired(res: &Response) -> Result<(), crate::Error<TokenExpired>> {
    if res.status() == StatusCode::FOUND {
        let location = res
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if location.contains("simple.jsp") || location.contains("login") {
            return Err(crate::Error::Other(TokenExpired));
        }
    }
    Ok(())
}

/// 课程列表页，区分单课程账号和多课程账号
#[derive(Debug)]
pub enum CourseListPage {
    /// 单课程账号，来自 `main.jsp` 的课程下拉菜单
    Main(String),
    /// 多课程账号，来自 `courselist.jsp` 的课程列表
    List(String),
}

/// 获取课程列表页
///
/// 单课程账号会从 `courselist.jsp` 重定向到 `main.jsp`，此时返回 [CourseListPage::Main]
pub async fn course_list(token: &CgToken) -> Result<CourseListPage, crate::Error<TokenExpired>> {
    let res = client
        .get(format!("{}{}", BASE_URL, COURSELIST_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;
    check_token_expired(&res)?;

    match res.status() {
        StatusCode::FOUND => Ok(CourseListPage::Main(main_page(token).await?)),
        StatusCode::OK => Ok(CourseListPage::List(res.text().await.unexpected_err()?)),
        _ => Err(format!("获取课程列表失败: HTTP {}", res.status())).unexpected_err(),
    }
}

/// 进入课程上下文，跟随重定向以建立服务端 session 状态
pub async fn enter_course_context(
    token: &CgToken,
    course_id: u32,
) -> Result<(), crate::Error<TokenExpired>> {
    let res = client
        .get(format!("{}{}", BASE_URL, COURSELIST_URL))
        .query(&[("courseID", course_id.to_string())])
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;
    check_token_expired(&res)?;

    if res.status() == StatusCode::FOUND {
        let location = res
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
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
pub async fn main_page(token: &CgToken) -> Result<String, crate::Error<TokenExpired>> {
    let res = client
        .get(format!("{}{}", BASE_URL, MAIN_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;
    check_token_expired(&res)?;
    res.text().await.unexpected_err()
}

/// 获取作业列表页 HTML
pub async fn assignment_list_page(token: &CgToken) -> Result<String, crate::Error<TokenExpired>> {
    let res = client
        .get(format!("{}{}", BASE_URL, ASSIGNMENT_LIST_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;
    check_token_expired(&res)?;
    res.text().await.unexpected_err()
}

/// 获取题目列表页 HTML
pub async fn problem_list_page(
    token: &CgToken,
    assign_id: u32,
) -> Result<String, crate::Error<TokenExpired>> {
    let res = client
        .get(format!("{}{}", BASE_URL, PROBLEM_LIST_URL))
        .query(&[("assignID", assign_id.to_string())])
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;
    check_token_expired(&res)?;
    res.text().await.unexpected_err()
}

/// 手动跟随重定向的最大跳数（client 全局禁重定向，见 utils::client）
const MAX_REDIRECTS: u32 = 5;

/// 从 URL 提取题型标识
///
/// 如 `https://cg.hnu.edu.cn/assignment/programList_ce.jsp?x=1` → `programList_ce`
fn page_type_of(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let file = path.rsplit('/').next().unwrap_or(path);
    let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
    stem.to_string()
}

/// 按 URL 获取页面，手动跟随重定向，返回 `(题型标识, 页面 HTML)`
async fn get_following_redirects(
    token: &CgToken,
    url: &str,
) -> Result<(String, String), crate::Error<TokenExpired>> {
    let mut current = if url.starts_with("http") {
        url.to_string()
    } else {
        format!("{}{}", BASE_URL, url)
    };

    for _ in 0..MAX_REDIRECTS {
        let res = client
            .get(&current)
            .headers(token.headers().clone())
            .send()
            .await
            .network_err()?;
        check_token_expired(&res)?;

        if !res.status().is_redirection() {
            let html = res.text().await.unexpected_err()?;
            return Ok((page_type_of(&current), html));
        }

        let location = res
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let base = reqwest::Url::parse(&current).unexpected_err()?;
        current = base.join(location).unexpected_err()?.to_string();
    }

    Err(format!("获取页面重定向超过 {MAX_REDIRECTS} 次: {url}")).unexpected_err()
}

/// 获取题目详情页，跟随重定向
///
/// `url` 取自列表页题目链接的 href，即 [CgProblem::url](super::CgProblem::url)
///
/// 返回 `(题型标识, 页面 HTML)`，题型标识为最终 URL（跟随重定向后）的文件名主干（去扩展名）
pub async fn problem_page(
    token: &CgToken,
    url: &str,
) -> Result<(String, String), crate::Error<TokenExpired>> {
    get_following_redirects(token, url).await
}
