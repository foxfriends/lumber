use lumber::{Lumber, Question};
use std::convert::TryFrom;
use std::io::{stdout, Write};
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
pub fn main(opts: Opts) {
    let program = match opts.module {
        Some(path) => match Lumber::from_file(path) {
            Ok(program) => program,
            Err(error) => {
                eprintln!("{}", error);
                return;
            }
        },
        None => Lumber::default(),
    };
    if opts.query.is_empty() {
        let mut query = String::new();
        loop {
            print!("?- ");
            stdout().flush().unwrap();
            let len = std::io::stdin().read_line(&mut query).unwrap();
            if len == 0 {
                break;
            }
            let query = std::mem::take(&mut query);
            answer(&program, &query);
        }
    } else {
        for query in &opts.query {
            answer(&program, &query);
        }
    }
}

fn answer(program: &Lumber, query: &str) {
    let query = query.trim().trim_end_matches('.');
    if query.is_empty() {
        return;
    }
    let question = match Question::try_from(query) {
        Ok(question) => question,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };
    let mut answers = program.ask(&question).peekable();
    if answers.peek().is_none() {
        println!("No answer.");
    } else {
        for binding in answers {
            let output = question
                .answer(&binding)
                .unwrap()
                .into_iter()
                .map(|(var, val)| {
                    format!(
                        "{} = {}",
                        var,
                        val.map(|val| val.to_string()).unwrap_or_else(|| "_".into())
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            if output.is_empty() {
                println!("Answered without bindings");
            } else {
                println!("{}", output);
            }
        }
    }
}
