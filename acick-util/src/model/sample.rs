use std::hash::Hash;
use std::vec::IntoIter;

use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sample {
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    input: String,
    #[get = "pub"]
    output: String,
}

impl Sample {
    pub fn new(
        name: impl Into<String>,
        input: impl Into<String>,
        output: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            input: input.into(),
            output: output.into(),
        }
    }

    pub fn take(self) -> (String, String, String) {
        (self.name, self.input, self.output)
    }
}

pub trait AsSamples: Iterator<Item = Result<Sample>> {
    fn len(&self) -> usize;

    fn max_name_len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone)]
pub struct SampleIter {
    len: usize,
    max_name_len: usize,
    iter: IntoIter<Sample>,
}

impl Iterator for SampleIter {
    type Item = Result<Sample>;

    fn next(&mut self) -> Option<Result<Sample>> {
        self.iter.next().map(Ok)
    }
}

impl AsSamples for SampleIter {
    fn len(&self) -> usize {
        self.len
    }

    fn max_name_len(&self) -> usize {
        self.max_name_len
    }
}

impl From<Vec<Sample>> for SampleIter {
    fn from(samples: Vec<Sample>) -> Self {
        Self {
            len: samples.len(),
            max_name_len: samples.iter().map(|s| s.name.len()).max().unwrap_or(0),
            iter: samples.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_iter() {
        let sample_iter: SampleIter = vec![
            Sample::new("name 1", "5", "0"),
            Sample::new("name long long", "5", "0"),
            Sample::new("name hoge", "5", "0"),
        ]
        .into();
        assert_eq!(sample_iter.len(), 3);
        assert_eq!(sample_iter.max_name_len(), 14);
        assert_eq!(sample_iter.is_empty(), false);

        let sample_iter: SampleIter = Vec::new().into();
        assert_eq!(sample_iter.len(), 0);
        assert_eq!(sample_iter.max_name_len(), 0);
        assert_eq!(sample_iter.is_empty(), true);
    }
}
