use std::io::{self, Write as _};

use anyhow::{anyhow, Context as _};
use dropbox_sdk::files::FileMetadata;
use itertools::Itertools as _;
use rayon::prelude::*;
use strum::IntoEnumIterator as _;

use crate::abs_path::AbsPathBuf;
use crate::dropbox::Dropbox;
use crate::model::{ContestId, Problem};
use crate::{Console, Result};

#[derive(AsRefStr, EnumIter, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum InOut {
    In,
    Out,
}

pub fn fetch_full(
    dropbox: &Dropbox,
    contest_id: &ContestId,
    problems: &[Problem],
    testcases_path: &AbsPathBuf,
    cnsl: &mut Console,
) -> Result<()> {
    static DBX_TESTCASES_URL: &str =
        "https://www.dropbox.com/sh/arnpe0ef5wds8cv/AAAk_SECQ2Nc6SVGii3rHX6Fa?dl=0";

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

    // list testcase files
    let folders = problems
        .iter()
        .cartesian_product(InOut::iter())
        .collect::<Vec<_>>();
    let components_arr: Vec<(&Problem, InOut, Vec<FileMetadata>)> = folders
        .into_par_iter()
        .map(|(problem, inout)| {
            let files = dropbox
                .list_all_files(
                    format!("/{}/{}/{}", folder.name, problem.id(), inout.as_ref()),
                    Some(DBX_TESTCASES_URL),
                )
                .context("Could not list testcase files on Dropbox")?;
            Ok((problem, inout, files))
        })
        .collect::<Result<Vec<_>>>()?;

    // flatten testcase files data
    let components: Vec<(&Problem, InOut, FileMetadata)> = components_arr
        .into_iter()
        .map(|(problem, inout, files)| files.into_iter().map(move |file| (problem, inout, file)))
        .flatten()
        .collect();

    // calculate total size
    let total_size = components.iter().map(|(_, _, file)| file.size).sum();

    // download and save testcase files
    let pb = cnsl.build_pb_bytes(total_size);
    components
        .into_par_iter()
        .try_for_each::<_, Result<()>>(|(problem, inout, file)| {
            let dbx_path = format!(
                "/{}/{}/{}/{}",
                folder.name,
                problem.id(),
                inout.as_ref(),
                file.name
            );
            let mut reader = dropbox.get_shared_link_file(DBX_TESTCASES_URL, dbx_path)?;
            let abs_path = testcases_path.join(inout.as_ref()).join(file.name);
            abs_path.save_pretty(
                |mut file| {
                    io::copy(&mut reader, &mut file).context("Could not save testcase to file")?;
                    Ok(())
                },
                true,
                Some(&testcases_path),
                None,
            )?;
            pb.inc(file.size);
            Ok(())
        })?;
    pb.finish();

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tempfile::tempdir;

    use super::*;
    use crate::dropbox::Token;
    use crate::model::Compare;

    fn get_test_problems() -> Vec<Problem> {
        vec![
            Problem::new(
                "C",
                "Linear Approximation",
                "arc100_a",
                Duration::from_secs(2),
                "1024 MB".parse().unwrap(),
                Compare::Default,
                vec![],
            ),
            Problem::new(
                "D",
                "Equal Cut",
                "arc100_b",
                Duration::from_secs(2),
                "1024 MB".parse().unwrap(),
                Compare::Default,
                vec![],
            ),
            Problem::new(
                "E",
                "Or Plus Max",
                "arc100_c",
                Duration::from_secs(2),
                "1024 MB".parse().unwrap(),
                Compare::Default,
                vec![],
            ),
            Problem::new(
                "F",
                "Colorful Sequences",
                "arc100_d",
                Duration::from_secs(2),
                "1024 MB".parse().unwrap(),
                Compare::Default,
                vec![],
            ),
        ]
    }

    #[test]
    fn test_fetch_full() -> Result<()> {
        let test_dir = tempdir()?;

        let dropbox = Dropbox::new(Token {
            access_token: env!("ACICK_DBX_ACCESS_TOKEN").to_owned(),
        });
        let contest_id = ContestId::from("arc100");
        let problems = get_test_problems();
        let testcases_path = AbsPathBuf::try_new(test_dir.path().to_owned())?;
        let mut cnsl = Console::buf();

        let result = fetch_full(
            &dropbox,
            &contest_id,
            &problems[0..2],
            &testcases_path,
            &mut cnsl,
        );

        let output_str = String::from_utf8(cnsl.take_buf().unwrap())?;
        eprintln!("{}", output_str);

        // TODO: check if testcase files exists

        result
    }
}
