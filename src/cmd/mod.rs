use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::Result;

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub enum Cmd {
    /// Shows current config
    Show,
}
