//! The main entry point for the `arith` command-line interpreter.
//!
//! This module handles command-line argument parsing using `clap`,
//! initializes logging, and dispatches to either the interactive REPL mode
//! or file processing mode based on the provided arguments.

use arith::repl::run_repl;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells};
use env_logger::{Builder, Env};
use log::LevelFilter;

use arith::filemode;

/// Command-line arguments for the `arith` interpreter.
#[derive(Parser)]
#[command(
    version,
    about = "A simple command-line arithmetic interpreter.",
    long_about = "arith is a lightweight and efficient command-line interpreter for arithmetic expressions, built with Rust. It provides a simple yet powerful way to evaluate mathematical expressions directly from your terminal, supporting basic operations, operator precedence, implicit multiplication, and an interactive Read-Eval-Print Loop (REPL)."
)]
struct Cli {
    /// Turn debugging information on.
    ///
    /// Can be specified multiple times to increase verbosity (e.g., `-d`, `-dd`).
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Specifies one or more files to process.
    ///
    /// If no files are specified, the interpreter will start in interactive REPL mode.
    #[arg(short, long)]
    files: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate shell completions
    Completion {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: shells::Shell,
    },
}

/// The main function of the `arith` interpreter.
///
/// It parses command-line arguments, sets up logging, and then either
/// starts the REPL or processes expressions from files.
///
/// # Returns
/// A `std::io::Result<()>` indicating success or an I/O error.
fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    Builder::from_env(Env::default().default_filter_or("debug")).init();
    log::set_max_level(LevelFilter::Debug);

    if let Some(command) = args.command {
        match command {
            Commands::Completion { shell } => {
                let mut cmd = Cli::command();
                let cmd_name = cmd.get_name().to_string();
                generate(shell, &mut cmd, cmd_name, &mut std::io::stdout());
                return Ok(());
            }
        }
    }

    if args.files.is_empty() {
        run_repl()
    } else {
        filemode::run_file_mode(args.files) // Call the new orchestrator
    }
}
