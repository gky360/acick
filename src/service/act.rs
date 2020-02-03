use crate::model::{Contest, ContestId, LangNameRef, Problem, ProblemId};
use crate::{Console, Result};

pub trait Act {
    fn login(&self, user: String, pass: String, cnsl: &mut Console) -> Result<bool>;

    fn fetch(
        &self,
        contest_id: &ContestId,
        problem_id: &Option<ProblemId>,
        cnsl: &mut Console,
    ) -> Result<(Contest, Vec<Problem>)>;

    fn submit(
        &self,
        contest_id: &ContestId,
        problem: &Problem,
        lang_name: LangNameRef,
        source: &str,
        cnsl: &mut Console,
    ) -> Result<()>;
}
