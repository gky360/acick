mod atcoder;
mod atcoder_page;
mod scrape;
mod serve;

pub use atcoder::AtcoderService;
pub use serve::Serve;

pub static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
