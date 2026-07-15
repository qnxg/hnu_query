use super::{CgAssignment, CgCourse, CgProblem};
use crate::cg::error::CgError;

use scraper::{Html, Selector};

/// 从多课程列表页 (`courselist.jsp`) 解析课程
///
/// `html` 为 [`super::fetch::main_page`] 在 HTTP 200 时返回的 HTML 数据
pub(super) fn courses_from_list(html: &str) -> Result<Vec<CgCourse>, crate::Error<CgError>> {
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
            Some(CgCourse { id, name })
        })
        .collect();

    if courses.is_empty() {
        return Err(crate::Error::Other(CgError::CourseNotFound));
    }
    Ok(dedup_by_id(courses))
}

/// 从 `main.jsp` 的课程下拉菜单中解析课程
///
/// `html` 为 [`super::fetch::main_page`] 返回的 HTML 数据
pub(super) fn courses_from_main(html: &str) -> Result<Vec<CgCourse>, crate::Error<CgError>> {
    let document = Html::parse_document(html);
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
            Some(CgCourse { id, name })
        })
        .collect();

    if courses.is_empty() {
        return Err(crate::Error::Other(CgError::CourseNotFound));
    }
    Ok(dedup_by_id(courses))
}

/// 从 `assignment/mainActiveAssigns.jsp` 的返回 HTML 中解析作业列表
///
/// `html` 为 [`super::fetch::assignment_list_page`] 返回的 HTML 数据
pub(super) fn assignments(html: &str) -> Result<Vec<CgAssignment>, crate::Error<CgError>> {
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
            Some(CgAssignment { id, name })
        })
        .collect();

    Ok(assignments)
}

/// 从 `assignment/index.jsp` 的题目列表页解析题目元数据
///
/// `html` 为 [`super::fetch::problem_list_page`] 返回的 HTML 数据
pub(super) fn problems(html: &str) -> Result<Vec<CgProblem>, crate::Error<CgError>> {
    let document = Html::parse_document(html);
    let row_sel = Selector::parse("table.table-striped tr").expect("题目表格行 CSS 选择器");
    let link_sel = Selector::parse("a[href]").expect("链接 CSS 选择器");
    let td_sel = Selector::parse("td").expect("td CSS 选择器");

    let problems: Vec<CgProblem> = document
        .select(&row_sel)
        .filter_map(|row| {
            let links: Vec<_> = row.select(&link_sel).collect();

            // 找 programList.jsp 链接 → pro_num + title
            let pro_link = links.iter().find(|el| {
                el.value()
                    .attr("href")
                    .is_some_and(|h| h.contains("programList.jsp"))
            })?;
            let href = pro_link.value().attr("href")?;
            let pro_num: u32 = href
                .split("proNum=")
                .nth(1)?
                .split('&')
                .next()?
                .parse()
                .ok()?;
            let title = pro_link
                .text()
                .collect::<Vec<_>>()
                .concat()
                .trim()
                .to_string();
            if title.is_empty() {
                return None;
            }

            // 找 judgeDetailsRedirect.jsp 链接 → problem_id
            let judge_link = links.iter().find(|el| {
                el.value()
                    .attr("href")
                    .is_some_and(|h| h.contains("judgeDetailsRedirect.jsp"))
            })?;
            let judge_href = judge_link.value().attr("href")?;
            let problem_id: u32 = judge_href
                .split("problemID=")
                .nth(1)?
                .split('&')
                .next()?
                .parse()
                .ok()?;

            // 从 <td> 中提取分值（第二个 <td>）
            let score: f64 = row
                .select(&td_sel)
                .nth(1)?
                .text()
                .collect::<Vec<_>>()
                .concat()
                .trim()
                .parse()
                .ok()?;

            Some(CgProblem {
                pro_num,
                problem_id,
                title,
                score,
            })
        })
        .collect();

    if problems.is_empty() {
        return Err(crate::Error::Other(CgError::AssignmentNotFound));
    }
    Ok(problems)
}

/// 按 id 去重，保留首次出现的记录
fn dedup_by_id(courses: Vec<CgCourse>) -> Vec<CgCourse> {
    let mut seen = std::collections::HashSet::new();
    courses.into_iter().filter(|c| seen.insert(c.id)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_courses_from_list() -> TestResult<()> {
        let html = include_str!("test_data/courselist.html");
        let courses = courses_from_list(html)?;
        assert!(!courses.is_empty(), "应解析出至少一门课程");
        let first = &courses[0];
        assert!(first.id > 0, "课程 ID 应大于 0");
        assert!(!first.name.is_empty(), "课程名称不应为空");
        Ok(())
    }

    #[test]
    fn test_parse_courses_from_list_dedup() -> TestResult<()> {
        // 包含重复课程的 HTML
        let html = include_str!("test_data/courselist_dup.html");
        let courses = courses_from_list(html)?;
        // 应去重：两个相同 ID 的链接只保留一个
        let ids: Vec<u32> = courses.iter().map(|c| c.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique_ids.len(), "课程 ID 不应重复");
        Ok(())
    }

    #[test]
    fn test_parse_courses_from_list_empty() -> TestResult<()> {
        let html = "<html><body>no courses here</body></html>";
        let result = courses_from_list(html);
        assert!(result.is_err(), "无课程时应返回错误");
        Ok(())
    }

    #[test]
    fn test_parse_courses_from_main() -> TestResult<()> {
        let html = include_str!("test_data/main_single_course.html");
        let courses = courses_from_main(html)?;
        assert!(!courses.is_empty(), "应解析出至少一门课程");
        let first = &courses[0];
        assert!(first.id > 0, "课程 ID 应大于 0");
        assert!(!first.name.is_empty(), "课程名称不应为空");
        Ok(())
    }

    #[test]
    fn test_parse_courses_from_main_empty() -> TestResult<()> {
        let html = "<html><body>no dropdown here</body></html>";
        let result = courses_from_main(html);
        assert!(result.is_err(), "无课程时应返回错误");
        Ok(())
    }

    #[test]
    fn test_parse_assignments() -> TestResult<()> {
        let html = include_str!("test_data/assignment_list.html");
        let assignments = assignments(html)?;
        assert!(!assignments.is_empty(), "应解析出至少一个作业");
        let first = &assignments[0];
        assert!(first.id > 0, "作业 ID 应大于 0");
        assert!(!first.name.is_empty(), "作业名称不应为空");
        Ok(())
    }

    #[test]
    fn test_parse_assignments_empty() -> TestResult<()> {
        let html = "<html><body>no assignments here</body></html>";
        let assignments = assignments(html)?;
        assert!(assignments.is_empty(), "无作业时应返回空列表");
        Ok(())
    }

    #[test]
    fn test_parse_problems() -> TestResult<()> {
        let html = include_str!("test_data/problem_list.html");
        let problems = problems(html)?;
        assert_eq!(problems.len(), 3, "应解析出 3 道题");

        // 第一题
        let first = &problems[0];
        assert_eq!(first.pro_num, 1);
        assert_eq!(first.problem_id, 20001);
        assert_eq!(first.title, "第一题");
        assert!(
            (first.score - 100.0).abs() < f64::EPSILON,
            "分数应为 100.00"
        );

        // 第二题
        let second = &problems[1];
        assert_eq!(second.pro_num, 2);
        assert_eq!(second.problem_id, 20002);
        assert_eq!(second.title, "第二题");
        assert!((second.score - 50.0).abs() < f64::EPSILON, "分数应为 50.00");

        // 第三题
        let third = &problems[2];
        assert_eq!(third.pro_num, 3);
        assert_eq!(third.problem_id, 20003);
        assert_eq!(third.title, "第三题");
        assert!((third.score - 75.50).abs() < f64::EPSILON, "分数应为 75.50");

        Ok(())
    }

    #[test]
    fn test_parse_problems_empty() -> TestResult<()> {
        let html = "<html><body>no problems here</body></html>";
        let result = problems(html);
        assert!(result.is_err(), "无题目时应返回错误");
        Ok(())
    }

    /// 验证题目详情页的 HTML 包含必要的表单元素，以便调用方进行后续处理
    #[test]
    fn test_problem_page_html_contains_required_elements() -> TestResult<()> {
        let html = include_str!("test_data/problem_page.html");
        assert!(html.contains("cgProblemContentClass"), "应包含题目内容区域");
        assert!(html.contains("uploadFORM"), "应包含提交表单");
        Ok(())
    }
}
