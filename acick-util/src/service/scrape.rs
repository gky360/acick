use std::str::FromStr;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};

use crate::abs_path::AbsPathBuf;
use crate::service::session::WithRetry as _;
use crate::{Console, Result};

/// Parses normal (hankaku) digits or zenkaku digits.
///
/// # Examples
///
/// ```
/// use acick_util::service::scrape::parse_zenkaku_digits;
///
/// /// success
/// assert_eq!(parse_zenkaku_digits::<i32>("0123"), Ok(123));
/// assert_eq!(parse_zenkaku_digits::<i32>("０１２３"), Ok(123));
///
/// /// failure
/// assert!(parse_zenkaku_digits::<i32>("01x23").is_err());
/// assert!(parse_zenkaku_digits::<i32>("０１あ２３").is_err());
/// assert!(parse_zenkaku_digits::<i32>("01２３").is_err());
/// ```
pub fn parse_zenkaku_digits<T: FromStr>(s: &str) -> std::result::Result<T, T::Err> {
    s.parse().or_else(|err| {
        if s.chars().all(|c| '０' <= c && c <= '９') {
            s.chars()
                .map(|c| char::from((u32::from(c) - u32::from('０') + u32::from('0')) as u8))
                .collect::<String>()
                .parse()
        } else {
            Err(err)
        }
    })
}

pub trait GetHtml {
    /// Returns a url from which we get html.
    fn url(&self) -> Result<Url>;

    /// Request html with http GET method.
    fn get_html(
        &self,
        client: &Client,
        cookies_path: &AbsPathBuf,
        retry_limit: usize,
        retry_interval: Duration,
        cnsl: &mut Console,
    ) -> Result<(StatusCode, Html)> {
        let res = client
            .get(self.url()?)
            .with_retry(client, cookies_path, retry_limit, retry_interval)
            .retry_send(cnsl)?;
        let status = res.status();
        let html = res.text().map(|text| Html::parse_document(&text))?;
        Ok((status, html))
    }
}

pub trait Scrape {
    /// Gets the underlying element
    fn elem(&self) -> ElementRef;

    /// Finds first element that matches `selector`.
    ///
    /// Returns `None` if no matches are found.
    fn find_first(&self, selector: &Selector) -> Option<ElementRef> {
        self.elem().select(selector).next()
    }

    /// Gets texts inside the underlying element as `String`.
    fn inner_text(&self) -> String {
        self.elem().text().collect()
    }
}

impl Scrape for ElementRef<'_> {
    fn elem(&self) -> ElementRef {
        *self
    }
}

#[cfg(test)]
mod tests {
    use reqwest::redirect::Policy;
    use scraper::Selector;
    use tempfile::tempdir;

    use crate::assert_matches;
    use crate::console::ConsoleConfig;

    use super::*;

    fn client() -> Client {
        Client::builder()
            .redirect(Policy::none()) // redirects manually
            .build()
            .unwrap()
    }

    #[test]
    fn test_parse_zenkaku_digits() -> anyhow::Result<()> {
        assert_eq!(parse_zenkaku_digits::<i32>("0123"), Ok(123));
        assert_eq!(parse_zenkaku_digits::<i32>("０１２３"), Ok(123));
        assert_matches!(parse_zenkaku_digits::<i32>("01x23") => Err(_));
        assert_matches!(parse_zenkaku_digits::<i32>("０１あ２３") => Err(_));
        assert_matches!(parse_zenkaku_digits::<i32>("01２３") => Err(_));
        Ok(())
    }

    #[test]
    fn test_get_html() -> anyhow::Result<()> {
        struct GoogleComPageBuilder {};
        impl GetHtml for GoogleComPageBuilder {
            fn url(&self) -> Result<Url> {
                Ok(Url::parse("http://google.com")?)
            }
        }

        let builder = GoogleComPageBuilder {};
        let test_dir = tempdir()?;
        let cookies_path = AbsPathBuf::try_new(&test_dir)?.join("cookies.json");
        let cnsl = &mut Console::sink(ConsoleConfig::default());
        let (actual_status, actual_html) =
            builder.get_html(&client(), &cookies_path, 4, Duration::from_secs(2), cnsl)?;

        let expected_status = StatusCode::from_u16(301).unwrap();
        let expected_html = Html::parse_document(
            r#"<HTML><HEAD><meta http-equiv="content-type" content="text/html;charset=utf-8">
<TITLE>301 Moved</TITLE></HEAD><BODY>
<H1>301 Moved</H1>
The document has moved
<A HREF="http://www.google.com/">here</A>.
</BODY></HTML>
"#,
        );

        assert_eq!(actual_status, expected_status);
        assert_eq!(actual_html, expected_html);
        Ok(())
    }

    #[test]
    fn test_find_first() -> anyhow::Result<()> {
        let tests = &[
            (
                Html::parse_fragment("<ul><li>Foo</li><li>Bar</li><li>Baz</li></ul>"),
                Some(String::from("<li>Foo</li>")),
            ),
            (Html::parse_fragment("<ul></ul>"), None),
        ];

        for (left, right) in tests {
            let elem = left.root_element();
            let actual = &elem
                .find_first(&Selector::parse("ul > li").unwrap())
                .map(|elem| elem.html());
            let expected = right;
            assert_eq!(actual, expected);
        }
        Ok(())
    }

    #[test]
    fn test_inner_text() -> anyhow::Result<()> {
        let tests = &[
            (
                Html::parse_fragment("<ul><li>Foo</li><li>Bar</li><li>Baz</li></ul>"),
                "FooBarBaz",
            ),
            (Html::parse_fragment("<div></div>"), ""),
        ];

        for (left, right) in tests {
            let actual = left.root_element().inner_text();
            let expected = *right;
            assert_eq!(actual, expected);
        }
        Ok(())
    }
}
