pub mod appointment;
pub mod error;
pub mod grade;
pub mod login;
mod raw;

#[cfg(test)]
mod test;

pub use appointment::get_appointment;
pub use grade::get_grade;
