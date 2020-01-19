use std::fmt;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::config::Config;
use crate::Result;

pub trait Outcome: fmt::Display + fmt::Debug {
    fn to_yaml(&self) -> Result<String>;
}

impl<T: fmt::Display + fmt::Debug + Serialize> Outcome for T {
    fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Could not serialize outcome to yaml")
    }
}

pub trait Run {
    fn run(&self) -> Result<Box<dyn Outcome>>;
}

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub enum Cmd {
    /// Shows current config
    Show(ShowOpt),
}

impl Run for Cmd {
    fn run(&self) -> Result<Box<dyn Outcome>> {
        match self {
            Self::Show(opt) => opt.run(),
        }
    }
}

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct ShowOpt {}

impl Run for ShowOpt {
    fn run(&self) -> Result<Box<dyn Outcome>> {
        let outcome = Config::load().context("Could not load config")?;
        Ok(Box::new(outcome))
    }
}
