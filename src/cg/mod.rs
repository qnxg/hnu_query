//! CG 系统题目形式多样且提交方式不一，库只负责认证 + 页面导航 + 提取通用元数据，不解析题目具体内容也不处理提交。

pub mod course;
pub mod error;
pub mod login;

#[cfg(test)]
mod test;

pub use course::{get_assignment_list, get_course_list, get_problem_list, get_problem_page};
