use structopt::StructOpt;

macro_rules! assert_match {
    ($a:expr => $b:pat) => {
        assert!(match $a {
            $b => true,
            _ => false,
        });
    };
}

#[test]
fn run_with_no_args() {
    let args = [""];
    let res = acick::Opt::from_iter_safe(&args);
    assert_match!(res => Err(_));
}
