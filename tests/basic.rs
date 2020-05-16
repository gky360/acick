use std::fs;
use std::io::Write as _;

use lazy_static::lazy_static;
use structopt::StructOpt;
use tempfile::{tempdir, TempDir};

use acick_util::abs_path::AbsPathBuf;
use acick_util::assert_matches;

static ARC100_C_SOURCE: &str = r#"/*
[arc100] C - Linear Approximation
*/

#include <algorithm>
#include <iostream>
using namespace std;
typedef long long int ll;
typedef pair<int, int> pii;
typedef pair<ll, int> pli;

const int MAX_N = 200000;

int N;
int A[MAX_N];

ll solve() {
    int b[MAX_N];
    for (int i = 0; i < N; i++) {
        b[i] = A[i] - i + 1;
    }
    sort(b, b + N);

    ll ans = 0;
    for (int i = 0; i < N; i++) {
        ans += abs(b[i] - b[N / 2]);
    }
    return ans;
}

int main() {
    cin >> N;
    for (int i = 0; i < N; i++) {
        cin >> A[i];
    }

    cout << solve() << endl;

    return 0;
}
"#;

lazy_static! {
    static ref ACICK_TEST_ENABLE_SUBMIT: bool = std::env::var("ACICK_TEST_ENABLE_SUBMIT")
        .map(|v| !(v.is_empty() || v == "false" || v == "0"))
        .unwrap_or(false);
}

fn get_opt_common(test_dir: &TempDir, args: &[&str]) -> Result<acick::Opt, structopt::clap::Error> {
    let base_dir = &test_dir.path().display().to_string();
    let mut cmd = vec!["acick", "--quiet", "--assume-yes", "--base-dir", &base_dir];
    cmd.extend_from_slice(args);
    acick::Opt::from_iter_safe(&cmd)
}

fn replace_cookies_path_in_conf(
    test_dir: &TempDir,
    cookies_path: &AbsPathBuf,
) -> anyhow::Result<()> {
    let conf_path = test_dir.path().join("acick.yaml");
    let conf_str = fs::read_to_string(&conf_path)?;

    // remove current cookies_path config
    let conf_str: String = conf_str
        .lines()
        .map(|line| {
            if line.starts_with("  cookies_path: ") {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // add new cookies_path config
    let mut new_conf = String::new();
    for line in conf_str.lines() {
        if line == "session:" {
            new_conf.push_str(line);
            new_conf.push_str(&format!("\n  cookies_path: {}\n", cookies_path));
        } else {
            new_conf.push_str(line);
            new_conf.push('\n');
        }
    }

    // write new config to file
    let mut file = fs::File::create(&conf_path)?;
    file.write_all(new_conf.as_bytes())?;

    Ok(())
}

#[test]
fn run_with_no_args() {
    let args = &["acick"];
    let res = acick::Opt::from_iter_safe(args);
    assert_matches!(res => Err(_));
}

#[test]
fn compare_readme_usage_with_help_message() {
    let args = &["acick", "--help"];
    let res = acick::Opt::from_iter_safe(args);
    let err = res.unwrap_err();
    let mut long_help_message = Vec::new();
    long_help_message.push("```");
    long_help_message.extend(err.message.lines());
    long_help_message.push("```");

    let readme_str = include_str!("../README.md");
    let mut i = 0;
    let mut is_usage = false;
    for line in readme_str.lines() {
        if !is_usage && line.contains("__ACICK_USAGE_BEGIN__") {
            is_usage = true;
        } else if is_usage && line.contains("__ACICK_USAGE_END__") {
            is_usage = false;
        } else if is_usage {
            assert_eq!(line, long_help_message[i]);
            i += 1;
        }
    }
    assert_eq!(i, long_help_message.len());
}

#[test]
fn test_basic_usage() -> anyhow::Result<()> {
    let test_dir = tempdir()?;

    assert_matches!(get_opt_common(&test_dir, &["help"]) => Err(_));

    // check that config file is not created yet
    assert_matches!(get_opt_common(&test_dir, &["show"])?.run() => Err(_));

    // create config file
    get_opt_common(&test_dir, &["init"])?.run()?;

    // set cookies_path to be under the test_dir
    let cookies_path = AbsPathBuf::try_new(&test_dir)?.join("cookies.json");
    replace_cookies_path_in_conf(&test_dir, &cookies_path)?;
    get_opt_common(&test_dir, &["show"])?.run()?;

    // check that use is not logged in
    assert_matches!(get_opt_common(&test_dir, &["me"])?.run() => Err(_));

    get_opt_common(&test_dir, &["login"])?.run()?;
    get_opt_common(&test_dir, &["me"])?.run()?;
    get_opt_common(&test_dir, &["fetch", "c", "--full", "--open"])?.run()?;

    // write source to file
    let mut file = fs::File::create(test_dir.path().join("atcoder/arc100/c/Main.cpp"))?;
    file.write_all(ARC100_C_SOURCE.as_bytes())?;

    get_opt_common(&test_dir, &["test", "c"])?.run()?;
    get_opt_common(&test_dir, &["test", "c", "--full"])?.run()?;
    if *ACICK_TEST_ENABLE_SUBMIT {
        get_opt_common(&test_dir, &["submit", "c", "--open"])?.run()?;
    }
    get_opt_common(&test_dir, &["logout"])?.run()?;

    // check that use is logged out
    assert_matches!(get_opt_common(&test_dir, &["me"])?.run() => Err(_));

    Ok(())
}
