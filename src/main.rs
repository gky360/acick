#![warn(clippy::all)]

use std::io::Write;
use std::{io, process};

use failure::Fallible;
use structopt::StructOpt;

use acick::Opt;

fn main() -> Fallible<()> {
    let code = {
        let opt = Opt::from_args();
        match acick::run(&opt) {
            Ok(_) => 0,
            Err(err) => {
                io::stdout().flush()?;
                eprintln!();
                err.print_full_message();
                1
            }
        }
    };
    process::exit(code)
}
