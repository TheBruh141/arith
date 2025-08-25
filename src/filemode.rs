//! This module provides functionality for running the `arith` interpreter in file mode.
//!
//! In file mode, the interpreter reads expressions from one or more specified files,
//! evaluates them, and prints the results or any encountered errors to the console.

use crate::executor::evaluate_lines;
use std::fs::read_to_string;
use std::path::Path;

/// Runs the `arith` interpreter in file mode, processing expressions from the given files.
///
/// For each file, it reads the content, evaluates all expressions within it using
/// `evaluate_lines`, and prints the results. Errors encountered during file reading
/// or expression evaluation are reported to `stderr`.
///
/// # Arguments
/// * `files` - A `Vec<String>` where each string is the path to an input file.
///
/// # Returns
/// A `std::io::Result<()>` which is `Ok(())` if all files were processed
/// (even if some expressions within files resulted in errors), or `Err` if
/// there was an I/O error (e.g., file not found, permission denied).
pub fn run_file_mode(files: Vec<String>) -> std::io::Result<()> {
    for file_path_str in files {
        let path = Path::new(&file_path_str);
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or(&file_path_str);

        log::info!("Processing file: {}", file_name);
        let content = read_to_string(&file_path_str)?;

        println!("--- Results from {} ---", file_name);

        let results = evaluate_lines(&content);

        for (idx, result) in results.into_iter().enumerate() { // Added enumerate for expression number
            match result {
                Ok((val, expr_str)) => println!("{} [{}]: {}", expr_str, idx + 1, val), // New format
                Err(e) => {
                    eprintln!("Error in {}: {}", file_name, e);
                }
            }
        }
        println!(); // Add a newline for separation between files
    }
    Ok(())
}