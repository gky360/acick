#![warn(clippy::all)]
#![cfg_attr(coverage, feature(no_coverage))]

#[macro_use]
extern crate strum;

use dirs::{data_local_dir, home_dir};
use lazy_static::lazy_static;

pub mod abs_path;
pub mod console;
mod macros;
pub mod model;
pub mod service;
pub mod web;

use crate::abs_path::AbsPathBuf;
use crate::console::Console;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

lazy_static! {
    pub static ref DATA_LOCAL_DIR: AbsPathBuf = {
        let path = data_local_dir()
            .unwrap_or_else(|| {
                home_dir()
                    .expect("Could not get home dir")
                    .join(".local")
                    .join("share")
            })
            .join("acick");
        AbsPathBuf::try_new(path).unwrap()
    };
}
