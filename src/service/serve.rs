use crate::cmd::LoginOutcome;
// use crate::model::{Problem, ProblemId};
use crate::Result;

pub trait Serve {
    fn login(&mut self, user: String, pass: String) -> Result<LoginOutcome>;
    // fn fetch_problems(&mut self, problem_id: Option<ProblemId>) -> Result<Vec<Problem>>;
}
