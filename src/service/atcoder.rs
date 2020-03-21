use std::io::Write as _;

use anyhow::{anyhow, Context as _};
use lazy_static::lazy_static;
use maplit::hashmap;
use reqwest::blocking::{Client, Response};
use reqwest::redirect::Policy;
use reqwest::{StatusCode, Url};

use crate::abs_path::AbsPathBuf;
use crate::config::SessionConfig;
use crate::dropbox::DbxAuthorizer;
use crate::model::{Contest, ContestId, LangNameRef, Problem, ProblemId};
use crate::service::atcoder_full::{fetch_full, TestcaseIter};
use crate::service::atcoder_page::{
    HasHeader as _, LoginPageBuilder, SettingsPageBuilder, SubmitPageBuilder, TasksPageBuilder,
    TasksPrintPageBuilder, BASE_URL,
};
use crate::service::scrape::{ExtractCsrfToken as _, ExtractLangId as _, HasUrl as _};
use crate::service::session::WithRetry as _;
use crate::service::{Act, ResponseExt as _};
use crate::web::open_in_browser;
use crate::{Config, Console, Error, Result};

lazy_static! {
    // Use option_env for builds on crates.io.
    // crates.io does not know these secrets.
    static ref DBX_APP_KEY: &'static str = {
        #[allow(clippy::option_env_unwrap)]
        option_env!("ACICK_DBX_APP_KEY").unwrap()
    };
    static ref DBX_APP_SECRET: &'static str = {
        #[allow(clippy::option_env_unwrap)]
        option_env!("ACICK_DBX_APP_SECRET").unwrap()
    };
}

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "-",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);
static DBX_REDIRECT_PORT: u16 = 4100;
static DBX_REDIRECT_PATH: &str = "/oauth2/callback";

#[derive(Debug)]
pub struct AtcoderActor<'a> {
    client: Client,
    session: &'a SessionConfig,
}

impl<'a> AtcoderActor<'a> {
    pub fn new(session: &'a SessionConfig) -> Self {
        let client = Client::builder()
            .referer(false)
            .redirect(Policy::none()) // redirects manually
            .user_agent(USER_AGENT)
            .timeout(Some(session.timeout()))
            .build()
            .expect("Could not setup client. \
                TLS backend cannot be initialized, or the resolver cannot load the system configuration.");
        AtcoderActor { client, session }
    }
}

impl AtcoderActor<'_> {
    fn problem_url(contest_id: &ContestId, problem: &Problem) -> Result<Url> {
        let path = format!("/contests/{}/tasks/{}", contest_id, &problem.url_name());
        BASE_URL
            .join(&path)
            .context(format!("Could not parse problem url : {}", path))
    }

    fn submissions_url(contest_id: &ContestId) -> Result<Url> {
        let path = format!("/contests/{}/submissions/me", contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse submissions url : {}", path))
    }

    fn validate_login_response(res: &Response) -> Result<()> {
        if res.status() != StatusCode::FOUND {
            return Err(Error::msg("Received invalid response code"));
        }
        Ok(())
    }

    fn validate_submit_response(res: &Response, contest_id: &ContestId) -> Result<()> {
        if res.status() != StatusCode::FOUND {
            return Err(Error::msg("Received invalid response code"));
        }
        let loc_url = res
            .location_url(&BASE_URL)
            .context("Could not extract redirection url from response")?;
        if loc_url != Self::submissions_url(contest_id)? {
            return Err(Error::msg("Found invalid redirection url"));
        }
        Ok(())
    }

    pub fn fetch_full(
        contest_id: &ContestId,
        problems: &[Problem],
        token_path: &AbsPathBuf,
        conf: &Config,
        cnsl: &mut Console,
    ) -> Result<()> {
        // authorize Dropbox account
        let dropbox = DbxAuthorizer::new(
            &DBX_APP_KEY,
            &DBX_APP_SECRET,
            DBX_REDIRECT_PORT,
            DBX_REDIRECT_PATH,
            &token_path,
        )
        .load_or_request(cnsl)?;

        fetch_full(&dropbox, contest_id, problems, conf, cnsl)
    }

    pub fn load_testcases(
        testcases_dir: AbsPathBuf,
        sample_name: &Option<String>,
    ) -> Result<TestcaseIter> {
        TestcaseIter::load(testcases_dir, sample_name)
    }
}

impl Act for AtcoderActor<'_> {
    fn login(&self, user: String, pass: String, cnsl: &mut Console) -> Result<bool> {
        let Self { client, session } = self;

        // check if user is already logged in
        let login_page = LoginPageBuilder::new(session).build(client, cnsl)?;
        if login_page.is_logged_in()? {
            let current_user = login_page.current_user()?;
            if current_user != user {
                return Err(anyhow!("Logged in as another user: {}", current_user));
            }
            return Ok(false);
        }

        // prepare payload
        let csrf_token = login_page.extract_csrf_token()?;
        let payload = hashmap!(
            "csrf_token" => csrf_token.as_str(),
            "username" => user.as_str(),
            "password" => pass.as_str(),
        );

        // post credentials
        let res = client
            .post(login_page.url()?)
            .form(&payload)
            .with_retry(client, session, cnsl)
            .retry_send()?;

        // check if login succeeded
        Self::validate_login_response(&res).context("Login rejected by service")?;
        let settings_page = SettingsPageBuilder::new(session).build(client, cnsl)?;
        let current_user = settings_page.current_user()?;
        if current_user != user {
            return Err(anyhow!("Logged in as another user: {}", current_user));
        }

        Ok(true)
    }

    fn fetch(
        &self,
        contest_id: &ContestId,
        problem_id: &Option<ProblemId>,
        cnsl: &mut Console,
    ) -> Result<(Contest, Vec<Problem>)> {
        let Self { client, session } = self;

        let tasks_page = TasksPageBuilder::new(contest_id, session).build(client, cnsl)?;
        let contest_name = tasks_page
            .extract_contest_name()
            .context("Could not extract contest name")?;
        let mut problems: Vec<Problem> = tasks_page
            .extract_problems()?
            .into_iter()
            .filter(|problem| {
                if let Some(problem_id) = problem_id {
                    problem.id() == problem_id
                } else {
                    true
                }
            })
            .collect();
        if problems.is_empty() {
            let err = if let Some(problem_id) = problem_id {
                Err(anyhow!(
                    "Could not find problem \"{}\" in contest {}",
                    problem_id,
                    contest_id
                ))
            } else {
                Err(anyhow!(
                    "Could not find any problems in contest {}",
                    contest_id
                ))
            };
            return err;
        }

        let tasks_print_page =
            TasksPrintPageBuilder::new(contest_id, session).build(client, cnsl)?;
        let mut samples_map = tasks_print_page.extract_samples_map()?;
        for problem in problems.iter_mut() {
            if let Some(samples) = samples_map.remove(&problem.id()) {
                problem.set_samples(samples);
            } else {
                // found problem on TasksPage but not found on TasksPrintPage
                return Err(anyhow!(
                    "Could not extract samples for problem : {}",
                    problem.id()
                ));
            }
        }

        let contest = Contest::new(contest_id.to_owned(), contest_name);
        Ok((contest, problems))
    }

    fn submit(
        &self,
        contest_id: &ContestId,
        problem: &Problem,
        lang_name: LangNameRef,
        source: &str,
        cnsl: &mut Console,
    ) -> Result<()> {
        let Self { client, session } = self;

        // prepare payload
        let submit_page = SubmitPageBuilder::new(contest_id, session).build(client, cnsl)?;
        let csrf_token = submit_page.extract_csrf_token()?;
        let lang_id = submit_page.extract_lang_id(lang_name)?;
        let payload = hashmap!(
            "csrf_token" => csrf_token.as_str(),
            "data.TaskScreenName" => problem.url_name().as_str(),
            "data.LanguageId" => lang_id.as_str(),
            "sourceCode" => source,
        );

        // submit source code
        let res = client
            .post(submit_page.url()?)
            .form(&payload)
            .with_retry(client, session, cnsl)
            .retry_send()?;

        // check response
        Self::validate_submit_response(&res, contest_id)
            .context("Submission rejected by service")?;

        Ok(())
    }

    fn open_problem_url(
        &self,
        contest_id: &ContestId,
        problem: &Problem,
        cnsl: &mut Console,
    ) -> Result<()> {
        open_in_browser(&Self::problem_url(contest_id, problem)?.as_str())?;
        writeln!(cnsl, "Opened problem page in web browser.")?;
        Ok(())
    }

    fn open_submissions_url(&self, contest_id: &ContestId, cnsl: &mut Console) -> Result<()> {
        open_in_browser(&Self::submissions_url(contest_id)?.as_str())?;
        writeln!(cnsl, "Opened submissions page in web browser.")?;
        Ok(())
    }
}
