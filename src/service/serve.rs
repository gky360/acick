use crate::cmd::LoginOutcome;
use crate::Result;

pub trait Serve {
    fn login(&mut self, user: String, pass: String) -> Result<LoginOutcome>;
}
