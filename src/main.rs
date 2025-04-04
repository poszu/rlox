use std::{io::Write, path::Path};

use anyhow::{Context, Result};
use clap::Parser;
use rlox::{interpret::Interpreter, parse, pretty_printing::AstPrint, scanner::scan_tokens};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    script: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.script {
        Some(filepath) => {
            run_file(Path::new(&filepath))?;
        }
        None => run_prompt()?,
    }

    Ok(())
}

fn run_file(path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(path).context("reading source file")?;
    run(source)
}

fn run_prompt() -> Result<()> {
    let mut buffer = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let stdin = std::io::stdin();
        buffer.clear();
        stdin.read_line(&mut buffer)?;

        let tokens = match scan_tokens(&buffer) {
            Ok(tokens) => {
                println!("Scanned: '{tokens:?}'",);
                tokens
            }
            Err(err) => {
                println!("ERROR: {err}");
                continue;
            }
        };
        let parser = parse::Parser::new(tokens);
        let expr = match parser.parse() {
            Ok(expr) => {
                println!("Executing: '{}'", expr.print_ast());
                expr
            }
            Err(err) => {
                println!("ERROR: {err}");
                continue;
            }
        };
        let mut interpreter = Interpreter::default();
        match interpreter.evaluate(expr) {
            Ok(value) => {
                println!("{value}");
            }
            Err(err) => {
                println!("ERROR: {err}");
            }
        }
    }
}

fn run(source: String) -> Result<()> {
    for token in scan_tokens(&source)? {
        println!("New token: {:?}", token);
    }

    Ok(())
}
