use reqwest::blocking::Response;

use crate::service::scrape::{Accept, PageBase, Scrape};

pub type LoginPage = PageBase;

impl Accept<Response> for LoginPage {
    fn is_acceptable(&self, res: &Response) -> bool {
        res.status().is_success()
    }
}

impl Scrape for LoginPage {
    const HOST: &'static str = "https://atcoder.jp";
    const PATH: &'static str = "/login";
}
