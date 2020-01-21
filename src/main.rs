#![warn(clippy::all)]

use std::io;
use std::io::Write;

use structopt::StructOpt;

use acick::{Opt, Result};

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let (stdin, stdout, stderr) = (io::stdin(), io::stdout(), io::stderr());
    opt.run(stdin.lock(), stdout.lock(), stderr.lock())
        .map_err(|err| {
            io::stdout().flush().expect("Could not flush stdout");
            eprintln!();
            err
        })
}
