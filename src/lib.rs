#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use structopt::StructOpt;
use strum::VariantNames;

mod config;

use config::Config;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

#[derive(EnumString, EnumVariantNames, IntoStaticStr, Debug, Clone, PartialEq, Eq, Hash)]
#[strum(serialize_all = "kebab_case")]
pub enum ServiceKind {
    Atcoder,
}

#[derive(EnumString, EnumVariantNames, IntoStaticStr, Debug, Clone, PartialEq, Eq, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum LanguageKind {
    Cpp,
}

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Opt {
    #[structopt(
        long,
        global = true,
        env = "ACICK_SERVICE",
        default_value = ServiceKind::Atcoder.into(),
        possible_values = &ServiceKind::VARIANTS,
    )]
    service: ServiceKind,
    #[structopt(long, global = true, env = "ACICK_CONTEST", default_value = "abc100")]
    contest: String,
    #[structopt(
        long,
        global = true,
        env = "ACICK_LANGUAGE",
        default_value = LanguageKind::Cpp.into(),
        possible_values = &LanguageKind::VARIANTS
    )]
    language: LanguageKind,
    #[structopt(subcommand)]
    cmd: Cmd,
}

impl Opt {
    pub fn run(&self) -> Result<()> {
        eprintln!("{:?}", self);
        let conf = Config::load();
        eprintln!("{}", serde_yaml::to_string(&conf)?);
        println!("Hello, world!");
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
enum Cmd {
    /// Shows current config
    Show,
}
