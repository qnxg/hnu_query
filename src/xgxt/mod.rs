pub mod login;
pub mod personal_info;

#[cfg(test)]
mod test;

pub use personal_info::get_person_info;
