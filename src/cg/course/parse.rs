use super::{CgAssignment, CgCourse, CgProblem};
use scraper::{Html, Selector};
use std::sync::LazyLock;

// 编译期常量 CSS 选择器，避免每次调用重新解析
static COURSE_LIST_SEL: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("a[href*=\"courselist.jsp?courseID=\"]")
        .expect("courselist.jsp 课程链接 CSS 选择器")
});
static COURSE_MAIN_SEL: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("span.dropdown-item-course").expect("main.jsp 课程下拉菜单 CSS 选择器")
});
static ASSIGN_BLOCK_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div.main-zy").expect("作业列表块 CSS 选择器"));
static ASSIGN_TITLE_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("p.main-title").expect("作业标题 CSS 选择器"));
static ASSIGN_LINK_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a[href*=\"assignID=\"]").expect("作业链接 CSS 选择器"));
static PROBLEM_ROW_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("table.table-striped tr").expect("题目表格行 CSS 选择器"));
static PROBLEM_LINK_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a[href]").expect("链接 CSS 选择器"));
static PROBLEM_TD_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("td").expect("td CSS 选择器"));

/// 从多课程列表页 (`courselist.jsp`) 解析课程，页面中没有课程时返回 `None`
///
/// `html` 为 [`super::fetch::course_list`] 返回的 `CourseListPage::List` 中的 HTML
pub fn courses_from_list(html: &str) -> Option<Vec<CgCourse>> {
    let document = Html::parse_document(html);

    let courses: Vec<CgCourse> = document
        .select(&COURSE_LIST_SEL)
        .filter_map(|el| {
            let href = el.value().attr("href")?;
            let id: u32 = href
                .split("courselist.jsp?courseID=")
                .nth(1)?
                .parse()
                .ok()?;
            let name: String = el.text().collect();
            let name = name.trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(CgCourse { id, name })
        })
        .collect();

    if courses.is_empty() {
        return None;
    }
    Some(dedup_by_id(courses))
}

/// 从 `main.jsp` 的课程下拉菜单中解析课程，页面中没有课程时返回 `None`
///
/// `html` 为 [`super::fetch::course_list`] 返回的 `CourseListPage::Main` 中的 HTML
pub fn courses_from_main(html: &str) -> Option<Vec<CgCourse>> {
    let document = Html::parse_document(html);

    let courses: Vec<CgCourse> = document
        .select(&COURSE_MAIN_SEL)
        .filter_map(|el| {
            let id: u32 = el.value().attr("value")?.parse().ok()?;
            let name: String = el.text().collect();
            let name = name.trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(CgCourse { id, name })
        })
        .collect();

    if courses.is_empty() {
        return None;
    }
    Some(dedup_by_id(courses))
}

/// 从 `assignment/mainActiveAssigns.jsp` 的返回 HTML 中解析作业列表，没有作业时返回 `None`
///
/// `html` 为 [`super::fetch::assignment_list_page`] 返回的 HTML 数据
pub fn assignments(html: &str) -> Option<Vec<CgAssignment>> {
    let document = Html::parse_document(html);

    let assignments: Vec<CgAssignment> = document
        .select(&ASSIGN_BLOCK_SEL)
        .filter_map(|block| {
            let name: String = block.select(&ASSIGN_TITLE_SEL).next()?.text().collect();
            let name = name.trim().to_string();
            if name.is_empty() {
                return None;
            }
            let href = block
                .select(&ASSIGN_LINK_SEL)
                .next()?
                .value()
                .attr("href")?;
            let id: u32 = href.split("assignID=").nth(1)?.parse().ok()?;
            Some(CgAssignment { id, name })
        })
        .collect();

    if assignments.is_empty() {
        return None;
    }
    Some(assignments)
}

/// 从 `assignment/index.jsp` 的题目列表页解析题目元数据，没有题目时返回 `None`
///
/// `html` 为 [`super::fetch::problem_list_page`] 返回的 HTML 数据
pub fn problems(html: &str) -> Option<Vec<CgProblem>> {
    let document = Html::parse_document(html);

    let problems: Vec<CgProblem> = document
        .select(&PROBLEM_ROW_SEL)
        .filter_map(|row| {
            let links: Vec<_> = row.select(&PROBLEM_LINK_SEL).collect();

            // 找 programList.jsp 链接 → index + title
            let pro_link = links.iter().find(|el| {
                el.value()
                    .attr("href")
                    .is_some_and(|h| h.contains("programList.jsp"))
            })?;
            let href = pro_link.value().attr("href")?;
            let index: u32 = href
                .split("proNum=")
                .nth(1)?
                .split('&')
                .next()?
                .parse()
                .ok()?;
            let title: String = pro_link.text().collect();
            let title = title.trim().to_string();
            if title.is_empty() {
                return None;
            }

            // 找 judgeDetailsRedirect.jsp 链接 → id
            let judge_link = links.iter().find(|el| {
                el.value()
                    .attr("href")
                    .is_some_and(|h| h.contains("judgeDetailsRedirect.jsp"))
            })?;
            let judge_href = judge_link.value().attr("href")?;
            let id: u32 = judge_href
                .split("problemID=")
                .nth(1)?
                .split('&')
                .next()?
                .parse()
                .ok()?;

            // 从 <td> 中提取分值（第二个 <td>）
            let score_text: String = row.select(&PROBLEM_TD_SEL).nth(1)?.text().collect();
            let score: f64 = score_text.trim().parse().ok()?;

            Some(CgProblem {
                index,
                id,
                title,
                score,
            })
        })
        .collect();

    if problems.is_empty() {
        return None;
    }
    Some(problems)
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
        let courses = courses_from_list(html).expect("应解析出课程");
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
        let courses = courses_from_list(html).expect("应解析出课程");
        // 应去重：两个相同 ID 的链接只保留一个
        let ids: Vec<u32> = courses.iter().map(|c| c.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique_ids.len(), "课程 ID 不应重复");
        Ok(())
    }

    #[test]
    fn test_parse_courses_from_list_empty() -> TestResult<()> {
        let html = "<html><body>no courses here</body></html>";
        assert!(courses_from_list(html).is_none(), "无课程时应返回 None");
        Ok(())
    }

    #[test]
    fn test_parse_courses_from_main() -> TestResult<()> {
        let html = include_str!("test_data/main_single_course.html");
        let courses = courses_from_main(html).expect("应解析出课程");
        assert!(!courses.is_empty(), "应解析出至少一门课程");
        let first = &courses[0];
        assert!(first.id > 0, "课程 ID 应大于 0");
        assert!(!first.name.is_empty(), "课程名称不应为空");
        Ok(())
    }

    #[test]
    fn test_parse_courses_from_main_empty() -> TestResult<()> {
        let html = "<html><body>no dropdown here</body></html>";
        assert!(courses_from_main(html).is_none(), "无课程时应返回 None");
        Ok(())
    }

    #[test]
    fn test_parse_assignments() -> TestResult<()> {
        let html = include_str!("test_data/assignment_list.html");
        let assignments = assignments(html).expect("应解析出作业");
        assert!(!assignments.is_empty(), "应解析出至少一个作业");
        let first = &assignments[0];
        assert!(first.id > 0, "作业 ID 应大于 0");
        assert!(!first.name.is_empty(), "作业名称不应为空");
        Ok(())
    }

    #[test]
    fn test_parse_assignments_empty() -> TestResult<()> {
        let html = "<html><body>no assignments here</body></html>";
        assert!(assignments(html).is_none(), "无作业时应返回 None");
        Ok(())
    }

    #[test]
    fn test_parse_problems() -> TestResult<()> {
        let html = include_str!("test_data/problem_list.html");
        let problems = problems(html).expect("应解析出题目");
        assert_eq!(problems.len(), 3, "应解析出 3 道题");

        // 第一题
        let first = &problems[0];
        assert_eq!(first.index, 1);
        assert_eq!(first.id, 20001);
        assert_eq!(first.title, "第一题");
        assert!(
            (first.score - 100.0).abs() < f64::EPSILON,
            "分数应为 100.00"
        );

        // 第二题
        let second = &problems[1];
        assert_eq!(second.index, 2);
        assert_eq!(second.id, 20002);
        assert_eq!(second.title, "第二题");
        assert!((second.score - 50.0).abs() < f64::EPSILON, "分数应为 50.00");

        // 第三题
        let third = &problems[2];
        assert_eq!(third.index, 3);
        assert_eq!(third.id, 20003);
        assert_eq!(third.title, "第三题");
        assert!((third.score - 75.50).abs() < f64::EPSILON, "分数应为 75.50");

        Ok(())
    }

    #[test]
    fn test_parse_problems_empty() -> TestResult<()> {
        let html = "<html><body>no problems here</body></html>";
        assert!(problems(html).is_none(), "无题目时应返回 None");
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
