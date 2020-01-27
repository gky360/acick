use crate::cmd::LoginOutcome;
use crate::model::{Contest, ProblemId};
use crate::Result;

pub trait Serve {
    fn login(&mut self, user: String, pass: String) -> Result<LoginOutcome>;
    fn fetch(&mut self, problem_id: &Option<ProblemId>) -> Result<Contest>;
}
