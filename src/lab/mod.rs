pub mod course;
pub mod grade;
pub mod login;
pub mod schedule;
pub mod semester;

#[cfg(test)]
mod test;

pub use course::get_course_list;
pub use grade::{get_lab_grade, get_virtual_lab_grade};
pub use schedule::get_lab_schedule;
pub use semester::get_semester;
