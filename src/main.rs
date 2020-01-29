#![warn(clippy::all)]

use std::io;
use std::io::Write;

use structopt::StructOpt;

use acick::{Opt, Result};

fn main() -> Result<()> {
    let opt = Opt::from_args();
    if let Err(err) = opt.run(&mut io::stdin(), &mut io::stdout(), &mut io::stderr()) {
        io::stdout().flush().expect("Could not flush stdout");
        eprintln!();
        return Err(err);
    }
    Ok(())
}
