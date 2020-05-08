mod contest;
mod problem;
mod sample;
mod service;

pub use contest::*;
pub use problem::*;
pub use sample::*;
pub use service::*;

pub type LangId = String;

pub type LangIdRef<'a> = &'a str;

pub type LangName = String;

pub type LangNameRef<'a> = &'a str;
