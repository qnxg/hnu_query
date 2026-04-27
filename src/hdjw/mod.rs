pub mod class_table;
pub mod empty_classroom;
pub mod error;
pub mod exam_schedule;
pub mod grade;
pub mod login;
pub mod rank;
mod raw;

#[cfg(test)]
mod test;

pub use class_table::{get_class_table, get_class_table_extra};
pub use empty_classroom::get_empty_classroom;
pub use exam_schedule::get_exam_schedule;
pub use grade::{get_grade, get_grade_detail};
pub use rank::get_rank;
