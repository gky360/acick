mod atcoder;
mod atcoder_page;
mod request;
mod scrape;
mod serve;

pub use atcoder::AtcoderService;
pub use serve::Serve;

// TODO: use config
pub static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
