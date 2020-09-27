use lumber::Lumber;
use std::path::PathBuf;

/// Interactive Lumber (REPL)
#[derive(structopt::StructOpt)]
struct Opts {
    /// Query to be run, instead of opening the REPL.
    /// May be supplied multiple times.
    #[structopt(short, long)]
    query: Vec<String>,
    /// The main module of your program. Otherwise, opens the REPL with only standard definitions.
    module: Option<PathBuf>,
}

#[paw::main]
pub fn main(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let program = match opts.module {
        Some(path) => Lumber::from_file(path)?,
        None => Lumber::default(),
    };
    println!("{:?}", program);
    Ok(())
}
