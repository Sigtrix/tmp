use std::{
    fs, io,
    io::{BufRead, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

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
    /// Scan a Lox source file
    Scan {
        filename: PathBuf,
    },

    /// Parse a Lox source file and print its AST
    Parse {
        filename: PathBuf,
    },

    /// Execute a Lox program
    Exec {
        filename: PathBuf,
    },

    /// Start an interactive Lox REPL
    Repl,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Scan { filename } => tokenize(&filename),
        Command::Parse { filename } => parse(&filename),
        Command::Exec { filename } => evaluate(&filename),
        Command::Repl => repl(),
    }
}

fn tokenize(filename: &Path) -> ExitCode {
    let source = match read_source(filename) {
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

fn parse(filename: &Path) -> ExitCode {
    let source = match read_source(filename) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Failed to read {}: {error}", filename.display());
            return ExitCode::FAILURE;
        }
    };

    let statements = match parse_source(&source) {
        Ok(statements) => statements,
        Err(errors) => {
            print_errors(&errors);
            return ExitCode::FAILURE;
        }
    };

    for statement in &statements {
        println!("{statement}");
    }

    ExitCode::SUCCESS
}

fn evaluate(filename: &Path) -> ExitCode {
    let source = match read_source(filename) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Failed to read {}: {error}", filename.display());
            return ExitCode::FAILURE;
        }
    };

    let statements = match parse_source(&source) {
        Ok(statements) => statements,
        Err(errors) => {
            print_errors(&errors);
            return ExitCode::FAILURE;
        }
    };

    let interpreter = rlox_core::Interpreter;

    match interpreter.interpret(&statements) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn repl() -> ExitCode {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut stdout = io::stdout();

    let interpreter = rlox_core::Interpreter;
    let mut line = String::new();

    loop {
        print!("> ");

        if let Err(error) = stdout.flush() {
            eprintln!("Failed to flush prompt: {error}");
            return ExitCode::FAILURE;
        }

        line.clear();

        match input.read_line(&mut line) {
            Ok(0) => {
                println!();
                return ExitCode::SUCCESS;
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to read input: {error}");
                return ExitCode::FAILURE;
            }
        }

        if line.trim().is_empty() {
            continue;
        }

        let statements = match parse_source(&line) {
            Ok(statements) => statements,
            Err(errors) => {
                print_errors(&errors);
                continue;
            }
        };

        if let Err(error) = interpreter.interpret(&statements) {
            eprintln!("{error}");
        }
    }
}

fn read_source(filename: &Path) -> io::Result<String> {
    fs::read_to_string(filename)
}

fn parse_source(
    source: &str,
) -> Result<Vec<rlox_core::Stmt>, Vec<String>> {
    let result = rlox_core::Scanner::new(source).scan();

    if !result.errors.is_empty() {
        return Err(
            result
                .errors
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
    }

    let mut parser = rlox_core::Parser::new(result.tokens);

    parser
        .parse()
        .map_err(|error| vec![error.to_string()])
}

fn print_errors(errors: &[String]) {
    for error in errors {
        eprintln!("{error}");
    }
}