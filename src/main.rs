use arith::repl::run_repl;
use clap::Parser;
use env_logger::{Builder, Env};
use log::LevelFilter;

use arith::filemode;

#[derive(Parser)]
#[command(version, about, long_about = "... TODO!")]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[arg(short, long)]
    files: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    Builder::from_env(Env::default().default_filter_or("debug")).init();
    log::set_max_level(LevelFilter::Debug);

    if args.files.is_empty() {
        run_repl()
    } else {
        filemode::run_file_mode(args.files) // Call the new orchestrator
    }
}

