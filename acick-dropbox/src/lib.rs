#![warn(clippy::all)]

mod authorizer;
mod dropbox;
#[cfg_attr(coverage, no_coverage)]
mod hyper_client;

use acick_util::abs_path;
use acick_util::web;

pub use dropbox_sdk::files::FileMetadata;

pub use authorizer::{DbxAuthorizer, Token};
pub use dropbox::Dropbox;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

fn convert_dbx_err(err: dropbox_sdk::Error) -> Error {
    Error::msg(err.to_string())
}
