use anyhow::Context as _;
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{ElementRef, Html};

use crate::model::{LangId, LangIdRef, LangName, LangNameRef};
use crate::service::atcoder_page::{FetchRestricted, HasHeader, BASE_URL};
use crate::service::scrape::{
    select, ElementRefExt as _, ExtractCsrfToken, ExtractLangId, HasUrl, Scrape,
};
use crate::{Config, Console, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitPageBuilder<'a> {
    conf: &'a Config,
}

impl<'a> SubmitPageBuilder<'a> {
    pub fn new(conf: &'a Config) -> Self {
        Self { conf }
    }

    pub fn build(self, client: &Client, cnsl: &mut Console) -> Result<SubmitPage<'a>> {
        self.fetch_restricted(client, self.conf, cnsl)
            .map(|html| SubmitPage {
                builder: self,
                content: html,
            })
    }
}

impl HasUrl for SubmitPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        let contest_id = &self.conf.global_opt().contest_id;
        let path = format!("/contests/{}/submit", contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse url path: {}", path))
    }
}

impl FetchRestricted for SubmitPageBuilder<'_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitPage<'a> {
    builder: SubmitPageBuilder<'a>,
    content: Html,
}

impl SubmitPage<'_> {
    fn select_lang_options(&self) -> impl Iterator<Item = LangOptElem> {
        self.content
            .select(select!("#select-lang select option"))
            .map(LangOptElem)
    }
}

impl HasUrl for SubmitPage<'_> {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl Scrape for SubmitPage<'_> {
    fn elem(&self) -> ElementRef {
        self.content.root_element()
    }
}

impl HasHeader for SubmitPage<'_> {}

impl ExtractCsrfToken for SubmitPage<'_> {}

impl ExtractLangId for SubmitPage<'_> {
    fn extract_lang_id(&self, lang_name: LangNameRef) -> Result<LangId> {
        self.select_lang_options()
            .find_map(|opt| {
                if opt.extract_lang_name() == lang_name {
                    opt.extract_lang_id().map(Into::into)
                } else {
                    None
                }
            })
            .context(format!("Could not find language : {}", lang_name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LangOptElem<'a>(ElementRef<'a>);

impl LangOptElem<'_> {
    fn extract_lang_id(&self) -> Option<LangIdRef> {
        self.0.value().attr("value")
    }

    fn extract_lang_name(&self) -> LangName {
        self.0.inner_text()
    }
}
