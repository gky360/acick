use std::fs::read_dir;
use std::io::{self, Read as _, Write as _};
use std::path::Path;
use std::vec::IntoIter;

use anyhow::{anyhow, Context as _};
use rayon::prelude::*;
use strum::IntoEnumIterator as _;
use tempfile::tempdir;

use crate::abs_path::AbsPathBuf;
use crate::dropbox::{Dropbox, FileMetadata};
use crate::model::{AsSamples, ContestId, Problem, Sample};
use crate::{Config, Console, Error, Result};

static DBX_TESTCASES_URL: &str =
    "https://www.dropbox.com/sh/arnpe0ef5wds8cv/AAAk_SECQ2Nc6SVGii3rHX6Fa?dl=0";

#[derive(AsRefStr, EnumIter, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum InOut {
    In,
    Out,
}

impl InOut {
    fn par_iter() -> impl ParallelIterator<Item = Self> {
        Self::iter().collect::<Vec<_>>().into_par_iter()
    }
}

pub fn fetch_full(
    dropbox: &Dropbox,
    contest_id: &ContestId,
    problems: &[Problem],
    conf: &Config,
    cnsl: &mut Console,
) -> Result<()> {
    writeln!(cnsl, "Downloading testcase files from Dropbox ...")?;

    // find dropbox folder that corresponds to the contest
    let folders = dropbox.list_all_folders("", Some(DBX_TESTCASES_URL))?;
    let folder = folders
        .iter()
        .find(|folder| &ContestId::from(&folder.name) == contest_id)
        .ok_or_else(|| {
            anyhow!(
                "Could not find folder for the contest on Dropbox : {}",
                contest_id
            )
        })?;

    // download and save testcase files
    problems.iter().try_for_each(|problem| -> Result<()> {
        // setup temp dir
        let tmp_testcases_dir =
            tempdir().context("Could not create temp dir for downloading testcase files")?;
        let tmp_testcases_abs_dir = AbsPathBuf::try_new(tmp_testcases_dir.path().to_owned())?;

        // download testcase files for the problem
        fetch_problem_full(dropbox, &folder.name, problem, &tmp_testcases_abs_dir, cnsl)?;

        // move temp dir to testcases dir specified in config
        conf.move_testcases_dir(problem, &tmp_testcases_abs_dir, cnsl)?;

        Ok(())
    })
}

static TESTCASE_EXT: &str = "txt";

fn get_testcase_name(file_name: &str) -> Option<&str> {
    let file_path = Path::new(file_name);
    file_path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
}

/// Validates the file name of testcase and returns testcase name.
fn validate_testcase_file_name(file_name: &str) -> Option<&str> {
    let file_path = Path::new(file_name);
    let file_stem = file_path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str());
    let file_ext = file_path.extension().and_then(|file_ext| file_ext.to_str());

    if file_ext != Some(TESTCASE_EXT) {
        return None;
    }
    file_stem
}

fn get_testcase_file_name(testcase_name: &str) -> String {
    let mut file_name = String::from(testcase_name);
    file_name.push('.');
    file_name.push_str(TESTCASE_EXT);
    file_name
}

fn list_testcase_files(
    dropbox: &Dropbox,
    folder_name: &str,
    problem: &Problem,
) -> Result<Vec<(InOut, FileMetadata)>> {
    // fetch testcase files metadata
    let files_arr: Vec<(InOut, Vec<FileMetadata>)> = InOut::par_iter()
        .map(|inout| {
            let files = dropbox
                .list_all_files(
                    format!("/{}/{}/{}", folder_name, problem.id(), inout.as_ref()),
                    Some(DBX_TESTCASES_URL),
                )
                .context("Could not list testcase files on Dropbox")?;
            Ok((inout, files))
        })
        .collect::<Result<Vec<_>>>()?;

    // flatten testcase files metadata
    let files: Vec<(InOut, FileMetadata)> = files_arr
        .into_iter()
        .map(|(inout, files)| files.into_iter().map(move |file| (inout, file)))
        .flatten()
        .collect();
    Ok(files)
}

fn fetch_problem_full(
    dropbox: &Dropbox,
    folder_name: &str,
    problem: &Problem,
    testcases_dir: &AbsPathBuf,
    cnsl: &mut Console,
) -> Result<()> {
    let files = list_testcase_files(dropbox, folder_name, problem)?;

    // setup progress bar
    let total_size = files.iter().map(|(_, file)| file.size).sum();
    let pb = cnsl.build_pb_bytes(total_size);
    pb.set_prefix(problem.id().as_ref());

    // fetch and save
    files
        .into_par_iter()
        .try_for_each::<_, Result<()>>(|(inout, file)| {
            let dbx_path = format!(
                "/{}/{}/{}/{}",
                folder_name,
                problem.id(),
                inout.as_ref(),
                file.name
            );
            let mut reader = dropbox.get_shared_link_file(DBX_TESTCASES_URL, dbx_path)?;
            let testcase_name = get_testcase_name(&file.name)
                .ok_or_else(|| Error::msg("Failed to get testcase name from Dropbox file name"))?;
            let file_name = get_testcase_file_name(testcase_name);
            let abs_path = testcases_dir.join(inout.as_ref()).join(file_name);
            abs_path.save(
                |mut file| {
                    io::copy(&mut reader, &mut file).context("Could not save testcase to file")?;
                    Ok(())
                },
                true,
            )?;
            pb.inc(file.size);
            Ok(())
        })?;

    pb.finish();
    Ok(())
}

#[derive(Debug, Clone)]
pub struct TestcaseIter {
    dir: AbsPathBuf,
    len: usize,
    max_name_len: usize,
    names_iter: IntoIter<String>,
}

impl TestcaseIter {
    pub fn load(dir: AbsPathBuf, sample_name: &Option<String>) -> Result<Self> {
        let names = if let Some(sample_name) = sample_name {
            vec![sample_name.to_owned()]
        } else {
            let entries = read_dir(dir.join(InOut::In.as_ref()).as_ref())
                .context(
                    "Could not list testcase files. \
                     Download testcase files first by `acick fetch --full` command.",
                )?
                .collect::<io::Result<Vec<_>>>()?;
            let mut names = entries
                .iter()
                .filter(|entry| {
                    // check if entry is file
                    entry.file_type().map(|t| t.is_file()).unwrap_or(false)
                })
                .filter_map(|entry| {
                    let file_name = entry.file_name();
                    let file_name = file_name.to_string_lossy();
                    let testcase_name = validate_testcase_file_name(&file_name).map(Into::into);
                    testcase_name
                })
                .collect::<Vec<_>>();
            names.sort();
            names
        };

        let max_name_len = names.iter().map(|name| name.len()).max().unwrap_or(0);

        Ok(TestcaseIter {
            dir,
            len: names.len(),
            max_name_len,
            names_iter: names.into_iter(),
        })
    }

    fn load_file(&self, inout: InOut, testcase_name: &str) -> Result<String> {
        let file_name = get_testcase_file_name(testcase_name);
        let mut content = String::new();
        self.dir
            .join(inout.as_ref())
            .join(&file_name)
            .load(|mut file| {
                file.read_to_string(&mut content).with_context(|| {
                    format!(
                        "Could not load testcase {}put file: {}",
                        inout.as_ref(),
                        file_name
                    )
                })
            })?;
        Ok(content)
    }
}

impl Iterator for TestcaseIter {
    type Item = Result<Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        self.names_iter.next().map(|name| {
            let input = self.load_file(InOut::In, &name)?;
            let output = self.load_file(InOut::Out, &name)?;
            Ok(Sample::new(name, input, output))
        })
    }
}

impl AsSamples for TestcaseIter {
    fn len(&self) -> usize {
        self.len
    }

    fn max_name_len(&self) -> usize {
        self.max_name_len
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tempfile::tempdir;

    use super::*;
    use crate::console::ConsoleConfig;
    use crate::model::Compare;

    fn get_test_problems() -> Vec<Problem> {
        vec![
            Problem::new(
                "C",
                "Linear Approximation",
                "arc100_a",
                Some(Duration::from_secs(2)),
                Some("1024 MB".parse().unwrap()),
                Compare::Default,
                vec![],
            ),
            Problem::new(
                "D",
                "Equal Cut",
                "arc100_b",
                Some(Duration::from_secs(2)),
                Some("1024 MB".parse().unwrap()),
                Compare::Default,
                vec![],
            ),
            Problem::new(
                "E",
                "Or Plus Max",
                "arc100_c",
                Some(Duration::from_secs(2)),
                Some("1024 MB".parse().unwrap()),
                Compare::Default,
                vec![],
            ),
            Problem::new(
                "F",
                "Colorful Sequences",
                "arc100_d",
                Some(Duration::from_secs(2)),
                Some("1024 MB".parse().unwrap()),
                Compare::Default,
                vec![],
            ),
        ]
    }

    #[test]
    fn test_fetch_full() -> Result<()> {
        let test_dir = tempdir()?;

        let dropbox = Dropbox::from_access_token(std::env::var("ACICK_DBX_ACCESS_TOKEN").unwrap());
        let contest_id = ContestId::from("arc100");
        let problems = get_test_problems();
        let base_dir = AbsPathBuf::try_new(test_dir.path().to_owned()).unwrap();
        let conf = Config::default_in_dir(base_dir);
        let mut cnsl = Console::buf(ConsoleConfig::default());

        let result = fetch_full(&dropbox, &contest_id, &problems[0..1], &conf, &mut cnsl);
        let output_str = cnsl.take_output()?;
        eprintln!("{}", output_str);
        result?;

        let paths = &[
            "atcoder/arc100/c/testcases/in/sample_04.txt",
            "atcoder/arc100/c/testcases/in/subtask_1_11.txt",
            "atcoder/arc100/c/testcases/out/sample_04.txt",
            "atcoder/arc100/c/testcases/out/subtask_1_11.txt",
        ];
        for path in paths {
            assert!(test_dir.path().join(path).is_file());
        }

        Ok(())
    }

    #[test]
    fn test_get_testcase_name() {
        let fixture = &[
            ("", None),
            ("a", Some("a")),
            (".a", Some(".a")),
            (".a.txt", Some(".a")),
            ("a.txt", Some("a")),
        ];

        for (file_name, expected) in fixture {
            assert_eq!(get_testcase_name(file_name), *expected);
        }
    }

    #[test]
    fn test_validate_testcase_file_name() {
        let fixture = &[
            ("", None),
            ("a", None),
            (".a", None),
            (".a.txt", Some(".a")),
            ("a.txt", Some("a")),
        ];

        for (file_name, expected) in fixture {
            assert_eq!(validate_testcase_file_name(file_name), *expected);
            if let Some(testcase_name) = expected {
                assert_eq!(get_testcase_file_name(testcase_name), *file_name);
            }
        }
    }

    #[test]
    fn test_get_testcase_file_name() {
        assert_eq!(get_testcase_file_name("a"), "a.txt");
    }
}
