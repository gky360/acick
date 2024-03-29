#![warn(clippy::all)]
#![cfg_attr(coverage, feature(no_coverage))]

mod authorizer;
mod dropbox;

use acick_util::abs_path;
use acick_util::web;

pub use dropbox_sdk::files::FileMetadata;

pub use authorizer::DbxAuthorizer;
pub use dropbox::Dropbox;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

fn convert_dbx_err(err: dropbox_sdk::Error) -> Error {
    Error::msg(err.to_string())
}
