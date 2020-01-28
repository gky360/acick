#![warn(clippy::all)]

use std::io;
use std::io::Write;

use structopt::StructOpt;

use acick::{Opt, Result};

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let (stdin, stdout, stderr) = (io::stdin(), io::stdout(), io::stderr());
    if let Err(err) = opt.run(&mut stdin.lock(), &mut stdout.lock(), &mut stderr.lock()) {
        io::stdout().flush().expect("Could not flush stdout");
        eprintln!();
        return Err(err);
    }
    Ok(())
}
