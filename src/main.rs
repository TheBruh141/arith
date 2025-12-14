use ariadne::Source;
use arith::config::{Args, CliContext};
use arith::execute;
use clap::Parser;
use std::process::exit;
use std::fs;

fn main() {
    let args = Args::parse();

    //  Reconstruct Context for fancy errors
    let ctx = CliContext::new(&args.files);

    // Run Checks
    if let Err(report) = &args.check(&ctx) {
        report
            .print(("<command line>", Source::from(&ctx.full_command)))
            .unwrap();
        exit(1);
    }
    
    let files = args.files.clone(); // maybe move to arc?
    let comp_ops = &args.into_compiler_options();
    
    for path in files {
        println!("Compiling {}...", path.display());
        match fs::read_to_string(&path) {
            Ok(src) => execute(&src, path.to_str().unwrap_or("<unknown>"), comp_ops),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
