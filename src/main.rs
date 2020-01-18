#![warn(clippy::all)]

use std::io;
use std::io::Write;

use structopt::StructOpt;

use acick::{Opt, Result};

fn main() -> Result<()> {
    let opt = Opt::from_args();
    opt.run().map_err(|err| {
        io::stdout().flush().expect("Could not flush stdout");
        eprintln!();
        err
    })
}
