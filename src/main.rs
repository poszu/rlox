use std::{io::Write, path::Path};

use anyhow::Context;
use clap::Parser;
use rlox::scanner::scan_tokens;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    script: Option<String>,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.script {
        Some(filepath) => {
            println!("Running script {filepath}");
            run_file(Path::new(&filepath))?;
        }
        None => run_prompt()?,
    }

    Ok(())
}

fn run_file(path: &Path) -> Result<(), anyhow::Error> {
    let source = std::fs::read_to_string(path).context("Failed to read source file")?;
    run(source)?;

    Ok(())
}

fn run_prompt() -> Result<(), anyhow::Error> {
    let mut buffer = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let stdin = std::io::stdin();
        stdin.read_line(&mut buffer)?;

        let tokens = scan_tokens(&buffer)?;
        println!("Executing: '{:?}'", tokens.collect::<Vec<_>>());
        buffer.clear();
    }
}

fn run(source: String) -> Result<(), anyhow::Error> {
    for token in scan_tokens(&source)? {
        println!("New token: {:?}", token);
    }

    Ok(())
}
