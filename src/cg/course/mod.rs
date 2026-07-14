mod fetch;
mod parse;

use reqwest::StatusCode;
use reqwest::header::LOCATION;
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

/// CG 作业中的一道题目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgProblem {
    /// 题目序号（1-based，在作业中的编号）
    pub pro_num: u32,
    /// 题目 ID（提交代码时需要）
    pub problem_id: u32,
    /// 题目标题
    pub title: String,
    /// 分值
    pub score: f64,
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
            if location.contains("simple.jsp") || location.contains("login") {
                return Err(crate::Error::Other(CgError::TokenExpired));
            }
            // 单课程账号：重定向到 main.jsp，从 main.jsp 提取课程信息
            let body = fetch::main_page(token).await?;
            parse::parse_courses_from_main(&body)
        }
        StatusCode::OK => {
            // 多课程账号：解析课程列表页
            let body = res.text().await.network_err()?;
            parse::parse_courses_from_list(&body)
        }
        _ => Err(format!("获取课程列表失败: HTTP {}", res.status())).unexpected_err(),
    }
}

/// 获取指定课程的作业列表
///
/// 先进入课程上下文，再获取该课程的在线作业。
///
/// 如果传入无效的 `course_id`，服务器通常仍会重定向到 `main.jsp`，
/// 但不会返回任何作业内容，调用方会得到空列表。
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
pub async fn get_assignment_list(
    token: &CgToken,
    course_id: u32,
) -> Result<Vec<CgAssignment>, crate::Error<CgError>> {
    fetch::enter_course_context(token, course_id).await?;
    let body = fetch::assignment_list_page(token).await?;
    parse::parse_assignments(&body)
}

/// 获取作业的题目列表
///
/// 进入课程上下文后，解析 `assignment/index.jsp?assignID=xx` 页面
/// 提取题目元数据。
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
/// - [CgError::AssignmentNotFound] — 未找到题目
pub async fn get_problem_list(
    token: &CgToken,
    course_id: u32,
    assign_id: u32,
) -> Result<Vec<CgProblem>, crate::Error<CgError>> {
    fetch::enter_course_context(token, course_id).await?;
    let body = fetch::problem_list_page(token, assign_id).await?;
    parse::parse_problems(&body)
}

/// 获取题目详情页的原始 HTML
///
/// 进入课程上下文后，访问 `assignment/programList.jsp`，
/// 跟随 302 重定向到 `programList_ce.jsp`，返回完整 HTML。
/// 调用者自行解析题目描述、提交表单等处理。
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
pub async fn get_problem_page(
    token: &CgToken,
    course_id: u32,
    assign_id: u32,
    pro_num: u32,
) -> Result<String, crate::Error<CgError>> {
    fetch::enter_course_context(token, course_id).await?;
    fetch::problem_page(token, assign_id, pro_num).await
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

    #[tokio::test]
    #[ignore]
    async fn test_get_problem_list() -> TestResult<()> {
        let token = get_cg_token().await?;
        let courses = get_course_list(&token).await?;
        let course = courses.first().expect("至少有一个课程");
        let assignments = get_assignment_list(&token, course.course_id).await?;
        let assign = assignments.first().expect("至少有一个作业");

        println!("作业: [{}] {}", assign.assign_id, assign.assign_name);
        let problems = get_problem_list(&token, course.course_id, assign.assign_id).await?;
        println!("共 {} 道题:", problems.len());
        for p in &problems {
            println!(
                "  #{:3}  pid={:5}  score={:8.2}  {}",
                p.pro_num, p.problem_id, p.score, p.title
            );
        }
        assert!(!problems.is_empty(), "题目列表不应为空");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_problem_page() -> TestResult<()> {
        let token = get_cg_token().await?;
        let courses = get_course_list(&token).await?;
        let course = courses.first().expect("至少有一个课程");
        let assignments = get_assignment_list(&token, course.course_id).await?;
        let assign = assignments.first().expect("至少有一个作业");
        let problems = get_problem_list(&token, course.course_id, assign.assign_id).await?;
        let problem = problems.first().expect("至少有一道题");

        println!("获取题目页面: #{}/{}", problem.pro_num, problem.problem_id);
        let html =
            get_problem_page(&token, course.course_id, assign.assign_id, problem.pro_num).await?;
        println!("页面大小: {} bytes", html.len());
        assert!(!html.is_empty(), "题目页面不应为空");
        assert!(
            html.contains("cgProblemContentClass") || html.contains("problemID"),
            "页面应包含题目内容"
        );
        Ok(())
    }
}
