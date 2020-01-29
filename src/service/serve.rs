use crate::model::{Contest, ProblemId};
use crate::{Console, Result};

pub trait Serve {
    fn login(&self, user: String, pass: String, cnsl: &mut Console) -> Result<bool>;
    fn fetch(&self, problem_id: &Option<ProblemId>, cnsl: &mut Console) -> Result<Contest>;
}
