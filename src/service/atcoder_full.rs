use std::io::{self, Write as _};

use anyhow::{anyhow, Context as _};
use dropbox_sdk::files::FileMetadata;
use itertools::Itertools as _;
use rayon::prelude::*;

use crate::abs_path::AbsPathBuf;
use crate::dropbox::Dropbox;
use crate::model::{ContestId, Problem};
use crate::{Console, Result};

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
        .cartesian_product(&["in", "out"])
        .collect::<Vec<_>>();
    let components_arr: Vec<(&Problem, &str, Vec<FileMetadata>)> = folders
        .into_par_iter()
        .map(|(problem, inout)| {
            let files = dropbox
                .list_all_files(
                    format!("/{}/{}/{}", folder.name, problem.id(), inout),
                    Some(DBX_TESTCASES_URL),
                )
                .context("Could not list testcase files on Dropbox")?;
            Ok((problem, *inout, files))
        })
        .collect::<Result<Vec<_>>>()?;

    // flatten testcase files data
    let components: Vec<(&Problem, &str, FileMetadata)> = components_arr
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
            let dbx_path = format!("/{}/{}/{}/{}", folder.name, problem.id(), inout, file.name);
            let mut reader = dropbox.get_shared_link_file(DBX_TESTCASES_URL, dbx_path)?;
            let abs_path = testcases_path.join(inout).join(file.name);
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
