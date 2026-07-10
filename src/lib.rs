#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(rustdoc::all)]
#![warn(clippy::allow_attributes)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::too_long_first_doc_paragraph)]
#![warn(clippy::needless_pass_by_ref_mut)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::needless_collect)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::option_as_ref_deref)]
#![warn(clippy::redundant_pub_crate)]
#![warn(clippy::map_unwrap_or)]
#![warn(clippy::implicit_clone)]
#![warn(clippy::unused_async)]
#![warn(clippy::todo, reason = "在`git commit`之前，请确认代码中没有`todo!()`")]

pub mod ai;
pub mod ca;
pub mod cg;
mod error;
pub mod gym;
pub mod hdjw;
pub mod lab;
pub mod netflow;
pub mod pt;
mod utils;
pub mod wxpay;
pub mod xgxt;
pub mod yjsxt;
pub use error::Error;
pub mod cas;

#[cfg(test)]
pub mod test;
