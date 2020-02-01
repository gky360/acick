use std::cmp::max;

use getset::CopyGetters;
use itertools::{EitherOrBoth, Itertools as _};
use serde::{Deserialize, Serialize};

use crate::model::Compare;

#[derive(Serialize, Deserialize, CopyGetters, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextDiff {
    #[get_copy = "pub"]
    l_len: usize,
    #[get_copy = "pub"]
    r_len: usize,
    #[get_copy = "pub"]
    is_any: bool,
    left: String,
    right: String,
}

impl TextDiff {
    pub fn new(left: String, right: String, cmp: Compare) -> Self {
        let (l_len, r_len, is_any) =
            Self::lines(&left, &right).fold((0, 0, false), |(l_len, r_len, is_any), line| {
                (
                    max(l_len, line.0.len()),
                    max(r_len, line.1.len()),
                    is_any || !cmp.compare(line.0, line.1),
                )
            });

        Self {
            l_len,
            r_len,
            is_any,
            left,
            right,
        }
    }

    fn lines<'a>(left: &'a str, right: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        let (l_iter, r_iter) = (left.lines(), right.lines());
        l_iter.zip_longest(r_iter).map(|pair| match pair {
            EitherOrBoth::Both(l, r) => (l, r),
            EitherOrBoth::Left(l) => (l, ""),
            EitherOrBoth::Right(r) => ("", r),
        })
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a str, &'a str)> {
        Self::lines(&self.left, &self.right)
    }
}
