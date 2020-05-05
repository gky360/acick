use structopt::StructOpt;

use acick_util::assert_matches;

#[test]
fn run_with_no_args() {
    let args = ["acick"];
    let res = acick::Opt::from_iter_safe(&args);
    assert_matches!(res => Err(_));
}

#[test]
fn check_readme_usage() {
    let args = ["acick", "--help"];
    let res = acick::Opt::from_iter_safe(&args);
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
