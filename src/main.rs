#![warn(clippy::all)]

use std::io;
use std::io::Write;

use structopt::StructOpt;

use acick::{Context, Opt, Result};

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let (stdin, stdout, stderr) = (io::stdin(), io::stdout(), io::stderr());
    let mut ctx = Context::from_stdio(&stdin, &stdout, &stderr);
    opt.run(&mut ctx).map_err(|err| {
        io::stdout().flush().expect("Could not flush stdout");
        eprintln!();
        err
    })
}
