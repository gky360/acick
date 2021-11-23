use std::collections::BTreeMap;

use acick_util::{regex, select};
use anyhow::Context as _;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};

use crate::config::SessionConfig;
use crate::model::{ContestId, ProblemId, Sample};
use crate::page::{GetHtmlRestricted, BASE_URL};
use crate::service::scrape::{parse_zenkaku_digits, GetHtml, Scrape};
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
        self.get_html_restricted(client, self.session, cnsl)
            .map(|html| TasksPrintPage {
                builder: self,
                content: html,
            })
    }
}

impl GetHtml for TasksPrintPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        let path = format!("/contests/{}/tasks_print", self.contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse url path: {}", path))
    }
}

impl GetHtmlRestricted for TasksPrintPageBuilder<'_> {}

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
        static IN_OUT_REGEXS: &[(&Lazy<Regex>, &Lazy<Regex>)] = &[
            (
                regex!(r"\ASample Input\s?([0-9]{1,2}).*\z"),
                regex!(r"\ASample Output\s?([0-9]{1,2}).*\z"),
            ),
            (
                regex!(r"\A[\s\n]*入力例\s*(\d{1,2})[.\n]*\z"),
                regex!(r"\A[\s\n]*出力例\s*(\d{1,2})[.\n]*\z"),
            ),
        ];
        static PS: &[&Lazy<Selector>] = &[
            // Current style (Japanese)
            select!("span.lang > span.lang-ja > div.part > section > h3, span.lang > span.lang-ja > div.part > section > pre"),
            // Current style (English)
            select!("span.lang > span.lang-en > div.part > section > h3, span.lang > span.lang-en > div.part > section > pre"),
            // ARC019..ARC057 \ {ARC019/C, ARC046/D, ARC050, ARC052/{A, C}, ARC053, ARC055},
            // ABC007..ABC040 \ {ABC036}, ATC001, ATC002
            select!("div.part > section > h3, div.part > section > pre"),
            // ARC002..ARC018, ARC019/C, ABC001..ABC006
            select!("div.part > h3, div.part > section > pre"),
            // ARC001, dwacon2018-final/{A, B}
            select!("h3, section > pre"),
            // ARC046/D, ARC050, ARC052/{A, C}, ARC053, ARC055, ABC036, ABC041
            select!("section > h3, section > pre"),
            // ABC034
            select!("span.lang > span.lang-ja > section > h3, span.lang > span.lang-ja > section > pre"),
            // practice contest (Japanese)
            select!("span.lang > span.lang-ja > div.part > h3, span.lang > span.lang-ja > div.part > section > pre"),
            // kupc2015
            select!("h3, pre"),
        ];

        for p in PS {
            for (re_in, re_out) in IN_OUT_REGEXS {
                if let Some(samples) = self.try_extract_samples(p, re_in, re_out) {
                    return samples;
                }
            }
        }
        return vec![];
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
        for elem in self.0.select(selector) {
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
