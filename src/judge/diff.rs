use std::cmp::max;
use std::fmt;

use getset::{CopyGetters, Getters};
use itertools::{EitherOrBoth, Itertools as _};
use serde::{Deserialize, Serialize};

use crate::model::Compare;

#[derive(Serialize, Deserialize, Getters, CopyGetters, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextDiff {
    #[get = "pub"]
    l_title: String,
    #[get = "pub"]
    r_title: String,
    #[get_copy = "pub"]
    l_width: usize,
    #[get_copy = "pub"]
    r_width: usize,
    #[get_copy = "pub"]
    is_any: bool,
    left: String,
    right: String,
    cmp: Compare,
}

impl TextDiff {
    pub fn new(
        l_title: impl Into<String>,
        r_title: impl Into<String>,
        left: String,
        right: String,
        cmp: Compare,
    ) -> Self {
        let (l_title, r_title) = (l_title.into(), r_title.into());
        let (l_width, r_width, is_any) = Self::iter_lines(&left, &right).fold(
            (l_title.len(), r_title.len(), false),
            |(l_width, r_width, is_any), line| {
                (
                    max(l_width, line.0.len()),
                    max(r_width, line.1.len()),
                    is_any || !cmp.compare(line.0, line.1),
                )
            },
        );

        Self {
            l_title,
            r_title,
            l_width,
            r_width,
            is_any,
            left,
            right,
            cmp,
        }
    }

    fn iter_lines<'a>(left: &'a str, right: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        let (l_iter, r_iter) = (left.lines(), right.lines());
        l_iter.zip_longest(r_iter).map(|pair| match pair {
            EitherOrBoth::Both(l, r) => (l, r),
            EitherOrBoth::Left(l) => (l, ""),
            EitherOrBoth::Right(r) => ("", r),
        })
    }

    fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        Self::iter_lines(&self.left, &self.right)
    }
}

impl fmt::Display for TextDiff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "  | {:l_width$} | {:r_width$} ",
            self.l_title,
            self.r_title,
            l_width = self.l_width,
            r_width = self.r_width
        )?;
        writeln!(
            f,
            "--+-{:-<l_width$}-+-{:-<r_width$}-",
            "",
            "",
            l_width = self.l_width,
            r_width = self.r_width
        )?;
        for line in self.iter() {
            writeln!(
                f,
                "{} | {:l_width$} | {:r_width$} ",
                if self.cmp.compare(line.0, line.1) {
                    " "
                } else {
                    ">"
                },
                line.0,
                line.1,
                l_width = self.l_width,
                r_width = self.r_width
            )?;
        }
        Ok(())
    }
}
