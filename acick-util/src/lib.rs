#![warn(clippy::all)]

#[macro_use]
extern crate strum;

pub mod abs_path;
pub mod console;
mod macros;
pub mod model;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;
