pub mod course;
pub mod error;
pub mod login;

#[cfg(test)]
mod test;

pub use course::{CgAssignment, CgCourse, get_assignment_list, get_course_list};
pub use login::{CgSession, CgToken};
