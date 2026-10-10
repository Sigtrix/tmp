use std::{fs, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rlox")]
#[command(about = "A Lox interpreter implemented in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Tokenize a Lox source file
    Tokenize {
        /// The Lox source file to tokenize
        filename: PathBuf,
    },

    /// Parse a Lox source file and print its AST
    Parse {
        /// The Lox source file to parse
        filename: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Tokenize { filename } => tokenize(&filename),
        Command::Parse { filename } => parse(&filename),
    }
}

fn tokenize(filename: &PathBuf) -> ExitCode {
    let source = match fs::read_to_string(filename) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Failed to read {}: {error}", filename.display());
            return ExitCode::FAILURE;
        }
    };

    let result = rlox_core::Scanner::new(&source).scan();

    for error in &result.errors {
        eprintln!("{error}");
    }

    for token in &result.tokens {
        let literal = token
            .literal
            .as_ref()
            .map_or_else(|| "null".to_string(), ToString::to_string);

        println!("{} {} {}", token.token_type, token.lexeme, literal);
    }

    if result.errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn parse(filename: &PathBuf) -> ExitCode {
    let source = match fs::read_to_string(filename) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Failed to read {}: {error}", filename.display());
            return ExitCode::FAILURE;
        }
    };

    let result = rlox_core::Scanner::new(&source).scan();

    if !result.errors.is_empty() {
        for error in &result.errors {
            eprintln!("{error}");
        }

        return ExitCode::FAILURE;
    }

    let mut parser = rlox_core::Parser::new(result.tokens);

    match parser.parse() {
        Ok(expr) => {
            println!("{expr}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}