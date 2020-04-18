#![warn(clippy::all)]

#[macro_use]
extern crate strum;

mod actor;
mod full;
mod page;

use acick_config as config;
use acick_dropbox as dropbox;
use acick_util::{abs_path, console, model, service, web};

use crate::config::Config;
use crate::console::Console;

pub use actor::AtcoderActor;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;
