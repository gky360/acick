use std::cmp::max;

use getset::CopyGetters;
use itertools::{EitherOrBoth, Itertools as _};

use crate::model::Compare;

#[derive(CopyGetters, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextDiff<'a> {
    lines: Vec<LineDiff<'a>>,
    #[get_copy = "pub"]
    l_len: usize,
    #[get_copy = "pub"]
    r_len: usize,
    #[get_copy = "pub"]
    is_any: bool,
}

impl<'a> TextDiff<'a> {
    pub fn new(left: &'a str, right: &'a str, cmp: Compare) -> Self {
        let (l_iter, r_iter) = (left.lines(), right.lines());
        let lines: Vec<_> = l_iter
            .zip_longest(r_iter)
            .map(|pair| match pair {
                EitherOrBoth::Both(l, r) => LineDiff(l, r),
                EitherOrBoth::Left(l) => LineDiff(l, ""),
                EitherOrBoth::Right(r) => LineDiff("", r),
            })
            .collect();

        let (l_len, r_len, is_any) =
            lines
                .iter()
                .fold((0, 0, false), |(l_len, r_len, is_any), line| {
                    (
                        max(l_len, line.0.len()),
                        max(r_len, line.1.len()),
                        is_any || !cmp.compare(line.0, line.1),
                    )
                });

        Self {
            lines,
            l_len,
            r_len,
            is_any,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LineDiff<'a>(&'a str, &'a str);
