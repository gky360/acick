use crate::Result;

mod atcoder;

pub use atcoder::AtcoderService;

pub trait Serve {
    fn login(&mut self, user: &str, pass: &str) -> Result<()>;
}
