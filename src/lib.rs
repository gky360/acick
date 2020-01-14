#[macro_use]
extern crate strum_macros;

use core::fmt;

use failure::{Backtrace, Context, Fail};
use structopt::StructOpt;
use strum::VariantNames;

pub type Result<T> = std::result::Result<T, self::Error>;

#[derive(Debug)]
pub struct Error {
    inner: Context<String>,
}

impl Error {
    pub fn print_full_message(&self) {
        for (i, cause) in Fail::iter_chain(self).enumerate() {
            let head = if i == 0 && self.cause().is_none() {
                "error: "
            } else if i == 0 {
                "    error: "
            } else {
                "caused by: "
            };
            eprint!("{}", head);
            for (i, line) in cause.to_string().lines().enumerate() {
                if i > 0 {
                    eprintln!("{}{}", " ".repeat(head.len()), line);
                } else {
                    eprintln!("{}", line);
                }
            }
        }
        let backtrace = self.backtrace().unwrap().to_string();
        if !backtrace.is_empty() {
            eprintln!("\n{}", backtrace);
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }
    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Error {
        Error {
            inner: Context::new(msg.into()),
        }
    }
}

impl From<Context<String>> for Error {
    fn from(inner: Context<String>) -> Error {
        Error { inner }
    }
}

impl From<Context<&'static str>> for Error {
    fn from(inner: Context<&'static str>) -> Error {
        Error {
            inner: inner.map(|s| s.to_string()),
        }
    }
}

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

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
enum Cmd {
    /// Shows current config
    Show,
}

pub fn run(opt: &Opt) -> Result<()> {
    eprintln!("{:?}", opt);
    println!("Hello, world!");
    Ok(())
}
