use core::fmt;

use failure::{Backtrace, Context, Fail};

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

// Allows adding more context via a &str
impl From<Context<&'static str>> for Error {
    fn from(inner: Context<&'static str>) -> Error {
        Error {
            inner: inner.map(|s| s.to_string()),
        }
    }
}

pub struct Opt {}

pub fn run(_opt: &Opt) -> Result<()> {
    println!("Hello, world!");
    Ok(())
}
