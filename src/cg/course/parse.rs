use scraper::{Html, Selector};

use super::{CgAssignment, CgCourse, CgProblem};
use crate::cg::error::CgError;

/// 从多课程列表页 (`courselist.jsp`) 解析课程
pub(super) fn parse_courses_from_list(html: &str) -> Result<Vec<CgCourse>, crate::Error<CgError>> {
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
pub(super) fn parse_courses_from_main(html: &str) -> Result<Vec<CgCourse>, crate::Error<CgError>> {
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
pub(super) fn parse_assignments(html: &str) -> Result<Vec<CgAssignment>, crate::Error<CgError>> {
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

/// 从 `assignment/index.jsp` 的题目列表页解析题目元数据
pub(super) fn parse_problems(html: &str) -> Result<Vec<CgProblem>, crate::Error<CgError>> {
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
    use crate::test::TestResult;

    #[test]
    fn test_parse_problems() -> TestResult<()> {
        let html = include_str!("test_data/problem_list.html");
        let problems = parse_problems(html)?;
        assert!(!problems.is_empty(), "应解析出至少一道题");
        let first = &problems[0];
        assert_eq!(first.pro_num, 1);
        assert!(first.problem_id > 0);
        assert!(!first.title.is_empty());
        assert!(first.score > 0.0);
        Ok(())
    }

    #[test]
    fn test_parse_problem_page_contains_content() {
        let html = include_str!("test_data/problem_page.html");
        assert!(html.contains("cgProblemContentClass"));
        assert!(html.contains("uploadFORM"));
        assert!(html.contains("cgsoucecode"));
    }
}
