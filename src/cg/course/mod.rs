mod fetch;
mod parse;

use crate::{
    cg::{error::CgError, login::CgToken},
    utils::obs::{fetch_time, parse_time, traced},
};

use serde::{Deserialize, Serialize};

/// CG 系统中的课程
#[derive(Debug, Clone, Serialize, Deserialize)]
#[expect(clippy::module_name_repetitions)]
pub struct CgCourse {
    /// 课程 ID
    pub id: u32,
    /// 课程名称
    pub name: String,
}

/// CG 系统中的作业
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgAssignment {
    /// 作业 ID
    pub id: u32,
    /// 作业名称
    pub name: String,
}

/// CG 作业中的一道题目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgProblem {
    /// 题目序号（1-based，在作业中的编号）
    pub index: u32,
    /// 题目 ID（提交代码时需要）
    pub id: u32,
    /// 题目标题
    pub title: String,
    /// 分值
    pub score: f64,
}

/// 获取当前账号的课程列表
///
/// # Arguments
///
/// - `token`: CG 系统的登录令牌，可以通过 [CgSession::login](crate::cg::login::CgSession::login) 获取
///
/// # Returns
///
/// 返回课程列表
///
/// 如果返回 `None`，说明当前账号下没有课程
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
#[traced(subsystem = "cg", skip(token))]
pub async fn get_course_list(
    token: &CgToken,
) -> Result<Option<Vec<CgCourse>>, crate::Error<CgError>> {
    // 单课程账号会从 `courselist.jsp` 重定向到 `main.jsp`，此时从 `main.jsp` 的课程下拉菜单提取
    let page = fetch_time!(fetch::course_list(token).await)?;
    let courses = match page {
        fetch::CourseListPage::Main(body) => parse_time!(parse::courses_from_main(&body)),
        fetch::CourseListPage::List(body) => parse_time!(parse::courses_from_list(&body)),
    };
    Ok(courses)
}

/// 获取指定课程的作业列表
///
/// # Arguments
///
/// - `token`: CG 系统的登录令牌，可以通过 [CgSession::login](crate::cg::login::CgSession::login) 获取
/// - `course_id`: 课程 ID，可通过 [get_course_list] 获取
///
/// # Returns
///
/// 返回作业列表
///
/// 如果返回 `None`，说明该课程没有作业
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
#[traced(subsystem = "cg", skip(token))]
pub async fn get_assignment_list(
    token: &CgToken,
    course_id: u32,
) -> Result<Option<Vec<CgAssignment>>, crate::Error<CgError>> {
    // 需要先进入课程上下文，服务端才会返回该课程的在线作业
    fetch_time!(fetch::enter_course_context(token, course_id).await)?;
    let body = fetch_time!(fetch::assignment_list_page(token).await)?;
    Ok(parse_time!(parse::assignments(&body)))
}

/// 获取作业的题目列表
///
/// # Arguments
///
/// - `token`: CG 系统的登录令牌，可以通过 [CgSession::login](crate::cg::login::CgSession::login) 获取
/// - `assign_id`: 作业 ID，可通过 [get_assignment_list] 获取
///
/// # Returns
///
/// 返回题目列表
///
/// 如果返回 `None`，说明该作业下没有题目，`assign_id` 可能无效
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
#[traced(subsystem = "cg", skip(token))]
pub async fn get_problem_list(
    token: &CgToken,
    assign_id: u32,
) -> Result<Option<Vec<CgProblem>>, crate::Error<CgError>> {
    let body = fetch_time!(fetch::problem_list_page(token, assign_id).await)?;
    Ok(parse_time!(parse::problems(&body)))
}

/// 获取题目详情页的原始 HTML
///
/// # Arguments
///
/// - `token`: CG 系统的登录令牌，可以通过 [CgSession::login](crate::cg::login::CgSession::login) 获取
/// - `assign_id`: 作业 ID
/// - `index`: 题目序号（1-based），可通过 [get_problem_list] 获取
///
/// # Returns
///
/// 返回题目详情页的完整 HTML 字符串，调用者自行解析题目描述、提交表单等
///
/// # Errors
///
/// - [CgError::TokenExpired] — 登录已过期
#[traced(subsystem = "cg", skip(token))]
pub async fn get_problem_page(
    token: &CgToken,
    assign_id: u32,
    index: u32,
) -> Result<String, crate::Error<CgError>> {
    // 访问 `assignment/programList.jsp` 后跟随 302 重定向到 `programList_ce.jsp` 才能拿到完整页面
    fetch_time!(fetch::problem_page(token, assign_id, index).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cg::test::get_cg_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_get_course_list() -> TestResult<()> {
        let token = get_cg_token().await?;
        let courses = get_course_list(&token).await?.expect("应获取到课程");
        println!("共 {} 门课程:", courses.len());
        for c in &courses {
            println!("  [{}] {}", c.id, c.name);
        }
        assert!(!courses.is_empty(), "课程列表不应为空");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_assignment_list() -> TestResult<()> {
        let token = get_cg_token().await?;
        let courses = get_course_list(&token).await?.expect("应获取到课程");
        println!("课程列表: {:?}", courses);
        if let Some(c) = courses.first() {
            println!("进入课程: {} ({})", c.name, c.id);
            let assignments = get_assignment_list(&token, c.id).await?;
            if let Some(assignments) = assignments {
                println!("共 {} 个作业:", assignments.len());
                for a in &assignments {
                    println!("  [{}] {}", a.id, a.name);
                }
            } else {
                println!("该课程没有作业");
            }
        }
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_problem_list() -> TestResult<()> {
        let token = get_cg_token().await?;
        let courses = get_course_list(&token).await?.expect("应获取到课程");
        let Some(course) = courses.first() else {
            eprintln!("没有课程，跳过测试");
            return Ok(());
        };
        let Some(assignments) = get_assignment_list(&token, course.id).await? else {
            eprintln!("没有作业，跳过测试");
            return Ok(());
        };
        let Some(assign) = assignments.first() else {
            eprintln!("没有作业，跳过测试");
            return Ok(());
        };

        println!("作业: [{}] {}", assign.id, assign.name);
        let problems = get_problem_list(&token, assign.id)
            .await?
            .expect("应获取到题目");
        println!("共 {} 道题:", problems.len());
        for p in &problems {
            println!(
                "  #{:3}  pid={:5}  score={:8.2}  {}",
                p.index, p.id, p.score, p.title
            );
        }
        assert!(!problems.is_empty(), "题目列表不应为空");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_problem_page() -> TestResult<()> {
        let token = get_cg_token().await?;
        let courses = get_course_list(&token).await?.expect("应获取到课程");
        let Some(course) = courses.first() else {
            eprintln!("没有课程，跳过测试");
            return Ok(());
        };
        let Some(assignments) = get_assignment_list(&token, course.id).await? else {
            eprintln!("没有作业，跳过测试");
            return Ok(());
        };
        let Some(assign) = assignments.first() else {
            eprintln!("没有作业，跳过测试");
            return Ok(());
        };
        let Some(problems) = get_problem_list(&token, assign.id).await? else {
            eprintln!("没有题目，跳过测试");
            return Ok(());
        };
        let Some(problem) = problems.first() else {
            eprintln!("没有题目，跳过测试");
            return Ok(());
        };

        println!("获取题目页面: #{}/{}", problem.index, problem.id);
        let html = get_problem_page(&token, assign.id, problem.index).await?;
        println!("页面大小: {} bytes", html.len());
        assert!(!html.is_empty(), "题目页面不应为空");
        assert!(
            html.contains("cgProblemContentClass") || html.contains("problemID"),
            "页面应包含题目内容"
        );
        Ok(())
    }
}
