use crate::model::{Contest, ContestId, LangName, LangNameRef, Problem, ProblemId};
use crate::{Console, Result};

pub trait Act {
    fn current_user(&self, cnsl: &mut Console) -> Result<Option<String>>;

    fn login(&self, user: String, pass: String, cnsl: &mut Console) -> Result<bool>;

    fn fetch(
        &self,
        contest_id: &ContestId,
        problem_id: &Option<ProblemId>,
        cnsl: &mut Console,
    ) -> Result<(Contest, Vec<Problem>)>;

    fn submit<'a>(
        &self,
        contest_id: &ContestId,
        problem: &Problem,
        lang_names: &'a [LangName],
        source: &str,
        cnsl: &mut Console,
    ) -> Result<LangNameRef<'a>>;

    fn open_problem_url(
        &self,
        contest_id: &ContestId,
        problem: &Problem,
        cnsl: &mut Console,
    ) -> Result<()>;

    fn open_submissions_url(&self, contest_id: &ContestId, cnsl: &mut Console) -> Result<()>;
}
