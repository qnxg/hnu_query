use reqwest::{StatusCode, header::LOCATION};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    cg::{CgToken, error::CgError},
    error::{MapNetworkErr, MapUnexpectedErr},
    utils::client,
};

const BASE_URL: &str = "https://cg.hnu.edu.cn";

/// CG 系统中的课程
#[derive(Debug, Clone, Serialize, Deserialize)]
#[expect(clippy::module_name_repetitions)]
pub struct CgCourse {
    /// 课程 ID
    pub course_id: u32,
    /// 课程名称
    pub course_name: String,
}

/// CG 系统中的作业
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgAssignment {
    /// 作业 ID
    pub assign_id: u32,
    /// 作业名称
    pub assign_name: String,
}

/// 获取当前账号的课程列表
///
/// 如果账号只有一个课程，CG 系统会从 `courselist.jsp` 直接重定向到 `main.jsp`，
/// 此时从 `main.jsp` 的课程下拉菜单中提取课程信息。
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
/// - [CgError::CourseNotFound] — 页面中未找到课程信息
pub async fn get_course_list(token: &CgToken) -> Result<Vec<CgCourse>, crate::Error<CgError>> {
    let res = client
        .get(format!("{}/courselist.jsp", BASE_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;

    match res.status() {
        StatusCode::FOUND => {
            let location = res
                .headers()
                .get(LOCATION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            // 重定向到登录页面说明 token 已过期
            if location.contains("simple.jsp") || location.contains("login") {
                return Err(crate::Error::Other(CgError::TokenExpired));
            }
            // 单课程账号：重定向到 main.jsp，从 main.jsp 提取课程信息
            parse_courses_from_main(token).await
        }
        StatusCode::OK => {
            // 多课程账号：解析课程列表页
            let body = res.text().await.network_err()?;
            parse_courses_from_list(&body)
        }
        _ => Err(format!("获取课程列表失败: HTTP {}", res.status())).unexpected_err(),
    }
}

/// 获取指定课程的作业列表
///
/// 先进入课程上下文（GET `courselist.jsp?courseID=xx`），
/// 再获取该课程的在线作业。
///
/// 如果传入无效的 `course_id`，服务器通常仍会重定向到 `main.jsp`，
/// 但不会返回任何作业内容，调用方会得到空列表，无法与「该课程确实没有作业」区分。
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
///
/// 没有作业时返回空列表。
pub async fn get_assignment_list(
    token: &CgToken,
    course_id: u32,
) -> Result<Vec<CgAssignment>, crate::Error<CgError>> {
    // 进入课程上下文
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
    } else {
        return Err(format!("进入课程失败: HTTP {}", res.status())).unexpected_err();
    }

    // 获取作业列表
    let res = client
        .get(format!("{}/assignment/mainActiveAssigns.jsp", BASE_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;

    let body = res.text().await.network_err()?;
    parse_assignments(&body)
}

// ========== 解析函数 ==========

/// 从多课程列表页 (`courselist.jsp`) 解析课程
fn parse_courses_from_list(html: &str) -> Result<Vec<CgCourse>, crate::Error<CgError>> {
    let document = Html::parse_document(html);
    let sel = Selector::parse("a[href*=\"courselist.jsp?courseID=\"]")
        .expect("courselist.jsp 课程链接 CSS 选择器");

    let courses: Vec<CgCourse> = document
        .select(&sel)
        .filter_map(|el| {
            let href = el.value().attr("href")?;
            let id: u32 = href
                .split("courselist.jsp?courseID=")
                .nth(1)?
                .parse()
                .ok()?;
            let name = el.text().collect::<Vec<_>>().concat().trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(CgCourse {
                course_id: id,
                course_name: name,
            })
        })
        .collect();

    if courses.is_empty() {
        return Err(crate::Error::Other(CgError::CourseNotFound));
    }
    Ok(dedup_by_id(courses))
}

/// 从 `main.jsp` 的课程下拉菜单中解析课程
async fn parse_courses_from_main(token: &CgToken) -> Result<Vec<CgCourse>, crate::Error<CgError>> {
    let res = client
        .get(format!("{}/main.jsp", BASE_URL))
        .headers(token.headers().clone())
        .send()
        .await
        .network_err()?;

    let body = res.text().await.network_err()?;
    let document = Html::parse_document(&body);
    let sel =
        Selector::parse("span.dropdown-item-course").expect("main.jsp 课程下拉菜单 CSS 选择器");

    let courses: Vec<CgCourse> = document
        .select(&sel)
        .filter_map(|el| {
            let id: u32 = el.value().attr("value")?.parse().ok()?;
            let name = el.text().collect::<Vec<_>>().concat().trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(CgCourse {
                course_id: id,
                course_name: name,
            })
        })
        .collect();

    if courses.is_empty() {
        return Err(crate::Error::Other(CgError::CourseNotFound));
    }
    Ok(dedup_by_id(courses))
}

/// 从 `assignment/mainActiveAssigns.jsp` 的返回 HTML 中解析作业列表
fn parse_assignments(html: &str) -> Result<Vec<CgAssignment>, crate::Error<CgError>> {
    let document = Html::parse_document(html);
    let block_sel = Selector::parse("div.main-zy").expect("作业列表块 CSS 选择器");
    let title_sel = Selector::parse("p.main-title").expect("作业标题 CSS 选择器");
    let link_sel = Selector::parse("a[href*=\"assignID=\"]").expect("作业链接 CSS 选择器");

    let assignments: Vec<CgAssignment> = document
        .select(&block_sel)
        .filter_map(|block| {
            let name = block
                .select(&title_sel)
                .next()?
                .text()
                .collect::<Vec<_>>()
                .concat()
                .trim()
                .to_string();
            let href = block.select(&link_sel).next()?.value().attr("href")?;
            let id: u32 = href.split("assignID=").nth(1)?.parse().ok()?;
            if name.is_empty() {
                return None;
            }
            Some(CgAssignment {
                assign_id: id,
                assign_name: name,
            })
        })
        .collect();

    Ok(assignments)
}

/// 按 course_id 去重，保留首次出现的记录
fn dedup_by_id(courses: Vec<CgCourse>) -> Vec<CgCourse> {
    let mut seen = std::collections::HashSet::new();
    courses
        .into_iter()
        .filter(|c| seen.insert(c.course_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cg::test::get_cg_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_course_list() -> TestResult<()> {
        let token = get_cg_token().await?;
        let courses = get_course_list(&token).await?;
        println!("共 {} 门课程:", courses.len());
        for c in &courses {
            println!("  [{}] {}", c.course_id, c.course_name);
        }
        assert!(!courses.is_empty(), "课程列表不应为空");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_assignment_list() -> TestResult<()> {
        let token = get_cg_token().await?;
        let courses = get_course_list(&token).await?;
        println!("课程列表: {:?}", courses);
        if let Some(c) = courses.first() {
            println!("进入课程: {} ({})", c.course_name, c.course_id);
            let assignments = get_assignment_list(&token, c.course_id).await?;
            println!("共 {} 个作业:", assignments.len());
            for a in &assignments {
                println!("  [{}] {}", a.assign_id, a.assign_name);
            }
        }
        Ok(())
    }
}
