use crate::model::{Contest, LangNameRef, Problem, ProblemId};
use crate::{Console, Result};

pub trait Act {
    fn login(&self, user: String, pass: String, cnsl: &mut Console) -> Result<bool>;

    fn fetch(
        &self,
        problem_id: &Option<ProblemId>,
        cnsl: &mut Console,
    ) -> Result<(Contest, Vec<Problem>)>;

    fn submit(
        &self,
        problem_id: &ProblemId,
        lang_name: LangNameRef,
        source: &str,
        cnsl: &mut Console,
    ) -> Result<()>;
}
