use std::collections::BTreeMap;

use anyhow::Context as _;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};

use crate::config::SessionConfig;
use crate::macros::{regex, select};
use crate::model::{ContestId, ProblemId, Sample};
use crate::service::atcoder_page::{FetchRestricted, BASE_URL};
use crate::service::scrape::{parse_zenkaku_digits, ElementRefExt as _, HasUrl, Scrape};
use crate::{Console, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksPrintPageBuilder<'a> {
    contest_id: &'a ContestId,
    session: &'a SessionConfig,
}

impl<'a> TasksPrintPageBuilder<'a> {
    pub fn new(contest_id: &'a ContestId, session: &'a SessionConfig) -> Self {
        Self {
            contest_id,
            session,
        }
    }

    pub fn build(self, client: &Client, cnsl: &mut Console) -> Result<TasksPrintPage<'a>> {
        self.fetch_restricted(client, self.session, cnsl)
            .map(|html| TasksPrintPage {
                builder: self,
                content: html,
            })
    }
}

impl HasUrl for TasksPrintPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        let path = format!("/contests/{}/tasks_print", self.contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse url path: {}", path))
    }
}

impl FetchRestricted for TasksPrintPageBuilder<'_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksPrintPage<'a> {
    builder: TasksPrintPageBuilder<'a>,
    content: Html,
}

impl TasksPrintPage<'_> {
    pub fn extract_samples_map(&self) -> Result<BTreeMap<ProblemId, Vec<Sample>>> {
        let mut samples_map = BTreeMap::new();
        for elem in self.select_problems() {
            let (id, _) = elem.extract_id_name()?;
            let samples = elem.select_statement()?.extract_samples();
            samples_map.insert(id, samples);
        }
        Ok(samples_map)
    }

    fn select_problems(&self) -> impl Iterator<Item = ProblemElem> {
        self.content
            .select(select!(
                "#main-container > .row > .col-sm-12:not(.next-page)"
            ))
            .map(ProblemElem)
    }
}

impl HasUrl for TasksPrintPage<'_> {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl Scrape for TasksPrintPage<'_> {
    fn elem(&self) -> ElementRef {
        self.content.root_element()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProblemElem<'a>(ElementRef<'a>);

impl ProblemElem<'_> {
    fn extract_id_name(&self) -> Result<(ProblemId, String)> {
        let title = self
            .find_first(select!(".h2"))
            .context("Could not find problem title")?
            .inner_text();
        let mut id_name = title.splitn(2, '-');
        let id = id_name.next().context("Could not find problem id")?.trim();
        let name = id_name
            .next()
            .context("Could not find problem name")?
            .trim();
        Ok((id.into(), name.to_owned()))
    }

    fn select_statement(&self) -> Result<StatementElem> {
        self.find_first(select!("#task-statement"))
            .context("Could not find task statement")
            .map(StatementElem)
    }
}

impl Scrape for ProblemElem<'_> {
    fn elem(&self) -> ElementRef {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StatementElem<'a>(ElementRef<'a>);

impl StatementElem<'_> {
    fn extract_samples(&self) -> Vec<Sample> {
        static IN_JA: &Lazy<Regex> = regex!(r"\A[\s\n]*入力例\s*(\d{1,2})[.\n]*\z");
        static OUT_JA: &Lazy<Regex> = regex!(r"\A[\s\n]*出力例\s*(\d{1,2})[.\n]*\z");
        static IN_EN: &Lazy<Regex> = regex!(r"\ASample Input\s?([0-9]{1,2}).*\z");
        static OUT_EN: &Lazy<Regex> = regex!(r"\ASample Output\s?([0-9]{1,2}).*\z");

        // Current style (Japanese)
        static P1: &Lazy<Selector> = select!(
            "span.lang > span.lang-ja > div.part > section > h3, span.lang > span.lang-ja > div.part > section > pre"
        );
        // Current style (English)
        static P2: &Lazy<Selector> = select!(
            "span.lang > span.lang-en > div.part > section > h3, span.lang > span.lang-en > div.part > section > pre"
        );
        // ARC019..ARC057 \ {ARC019/C, ARC046/D, ARC050, ARC052/{A, C}, ARC053, ARC055},
        // ABC007..ABC040 \ {ABC036}, ATC001, ATC002
        static P3: &Lazy<Selector> = select!("div.part > section > h3, div.part > section > pre");
        // ARC002..ARC018, ARC019/C, ABC001..ABC006
        static P4: &Lazy<Selector> = select!("div.part > h3, div.part > section > pre");
        // ARC001, dwacon2018-final/{A, B}
        static P5: &Lazy<Selector> = select!("h3, section > pre");
        // ARC046/D, ARC050, ARC052/{A, C}, ARC053, ARC055, ABC036, ABC041
        static P6: &Lazy<Selector> = select!("section > h3, section > pre");
        // ABC034
        static P7: &Lazy<Selector> = select!(
            "span.lang > span.lang-ja > section > h3, span.lang > span.lang-ja > section > pre"
        );
        // practice contest (Japanese)
        static P8: &Lazy<Selector> = select!(
            "span.lang > span.lang-ja > div.part > h3, span.lang > span.lang-ja > div.part > section > pre"
        );

        self.try_extract_samples(P1, IN_JA, OUT_JA)
            .or_else(|| self.try_extract_samples(P2, IN_EN, OUT_EN))
            .or_else(|| self.try_extract_samples(P3, IN_JA, OUT_JA))
            .or_else(|| self.try_extract_samples(P4, IN_JA, OUT_JA))
            .or_else(|| self.try_extract_samples(P5, IN_JA, OUT_JA))
            .or_else(|| self.try_extract_samples(P6, IN_JA, OUT_JA))
            .or_else(|| self.try_extract_samples(P7, IN_JA, OUT_JA))
            .or_else(|| self.try_extract_samples(P8, IN_JA, OUT_JA))
            .unwrap_or_default()
    }

    fn try_extract_samples(
        &self,
        selector: &'static Selector,
        re_input: &'static Regex,
        re_output: &'static Regex,
    ) -> Option<Vec<Sample>> {
        let mut inputs = BTreeMap::<usize, _>::new();
        let mut outputs = BTreeMap::<usize, _>::new();
        let mut next = None;
        for elem in self.0.select(&selector) {
            let elem_name = elem.value().name();
            if elem_name == "h3" {
                let text = elem.inner_text();
                if let Some(caps) = re_input.captures(&text) {
                    next = Some((true, parse_zenkaku_digits(&caps[1]).ok()?));
                } else if let Some(caps) = re_output.captures(&text) {
                    next = Some((false, parse_zenkaku_digits(&caps[1]).ok()?));
                }
            } else if ["pre", "section"].contains(&elem_name) {
                if let Some((is_input, n)) = next {
                    let text = elem.inner_text();
                    if is_input {
                        inputs.insert(n, text);
                    } else {
                        outputs.insert(n, text);
                    }
                }
                next = None;
            }
        }
        let mut samples = vec![];
        for (i, input) in inputs {
            if let Some(output) = outputs.remove(&i) {
                samples.push(Sample::new(i.to_string(), input, output));
            }
        }
        if samples.is_empty() {
            None
        } else {
            Some(samples)
        }
    }
}

impl Scrape for StatementElem<'_> {
    fn elem(&self) -> ElementRef {
        self.0
    }
}
