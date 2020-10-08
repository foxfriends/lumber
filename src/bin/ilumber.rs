use lumber::{Binding, Lumber, Question};
use std::convert::TryFrom;
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
    if opts.query.is_empty() {
        let mut query = String::new();
        while let Ok(len) = std::io::stdin().read_line(&mut query) {
            if len == 0 {
                break;
            }
            let query = std::mem::take(&mut query);
            let question = match Question::try_from(query.as_str()) {
                Ok(question) => question,
                Err(error) => {
                    eprintln!("{:?}", error);
                    continue;
                }
            };
            for answer in program.query::<Binding>(&question) {
                println!("{:?}", answer);
            }
        }
    } else {
        for query in &opts.query {
            let question = match Question::try_from(query.as_str()) {
                Ok(question) => question,
                Err(error) => {
                    eprintln!("{:?}", error);
                    continue;
                }
            };
            for answer in program.query::<Binding>(&question) {
                println!("{:?}", answer);
            }
        }
    }
    Ok(())
}
