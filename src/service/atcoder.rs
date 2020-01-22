use maplit::hashmap;
use once_cell::sync::OnceCell;
use reqwest::blocking::{Client, Response};
use scraper::{ElementRef, Html};

use crate::service::{select, Accept, Extract as _, LoginOutcome, Scrape, ScrapeOnce as _, Serve};
use crate::{Context, Result};

#[derive(Debug)]
pub struct AtcoderService<'a, 'b> {
    client: Client,
    ctx: &'a mut Context<'b>,
}

impl<'a, 'b> AtcoderService<'a, 'b> {
    pub fn new(client: Client, ctx: &'a mut Context<'b>) -> Self {
        Self { client, ctx }
    }
}

impl Serve for AtcoderService<'_, '_> {
    fn login(&mut self, user: &str, _pass: &str) -> Result<LoginOutcome> {
        let login_page = LoginPage::new();
        let elem = login_page.select_username_input(&self.client, self.ctx)?;
        eprintln!("{:?}", elem);

        let outcome = LoginOutcome {
            service_id: self.ctx.global_opt.service_id.clone(),
            username: user.to_string(),
        };
        Ok(outcome)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoginPage {
    content_cell: OnceCell<Html>,
}

impl LoginPage {
    fn new() -> Self {
        Self {
            content_cell: OnceCell::new(),
        }
    }

    fn select_username_input(&self, client: &Client, ctx: &mut Context) -> Result<ElementRef> {
        self.content(client, ctx)?.find_first(select!("#username"))
    }
}

impl Accept<Response> for LoginPage {
    fn is_acceptable(&self, res: &Response) -> bool {
        res.status().is_success()
    }
}

impl Scrape for LoginPage {
    const HOST: &'static str = "https://atcoder.jp";
    const PATH: &'static str = "/login";
}

impl AsRef<OnceCell<Html>> for LoginPage {
    fn as_ref(&self) -> &OnceCell<Html> {
        &self.content_cell
    }
}
