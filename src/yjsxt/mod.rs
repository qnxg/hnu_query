pub mod class_table;
pub mod error;
pub mod login;
pub mod term;
mod utils;

#[cfg(test)]
mod test;

pub use class_table::get_class_table;
pub use term::get_termcode;
