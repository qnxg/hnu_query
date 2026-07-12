pub mod class_table;
pub mod error;
mod fetch;
pub mod login;
mod parse;
pub mod semester;

#[cfg(test)]
mod test;

pub use class_table::get_class_table;
pub use semester::get_semester;
