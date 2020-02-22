#![warn(clippy::all)]

use std::io::{self, Write as _};

use structopt::StructOpt;

use acick::{Opt, Result};

fn main() -> Result<()> {
    let opt = Opt::from_args();
    if let Err(err) = opt.run() {
        io::stdout().flush().expect("Could not flush stdout");
        eprintln!();
        return Err(err);
    }
    Ok(())
}
