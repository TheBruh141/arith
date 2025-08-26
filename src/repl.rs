//! This module implements the Read-Eval-Print Loop (REPL) for the `arith` interpreter.
//!
//! The REPL provides an interactive command-line interface where users can
//! enter arithmetic expressions, and the interpreter will evaluate them and
//! print the results. It supports multi-line input, special commands, and
//! basic error reporting.

use crate::executor::evaluate_lines;
use log::error;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

/// Runs the interactive Read-Eval-Print Loop (REPL) for the `arith` interpreter.
///
/// This function continuously prompts the user for input, processes commands,
/// evaluates arithmetic expressions, and prints the results. It handles
/// line continuations, special REPL commands (like `:q`, `:help`, `:bench`, `:save`),
/// and displays evaluation errors.
///
/// # Returns
/// A `std::io::Result<()>` indicating success or an I/O error during input/output operations.
pub fn run_repl() -> io::Result<()> {
    println!("arith REPL — enter expressions. Use \\ for line-continuation. :q to quit.");

    let mut acc = String::new(); // accumulates current statement (may span lines)

    loop {
        // Primary prompt when empty, secondary when continuing
        if acc.is_empty() {
            print!(">> ");
        } else {
            print!("... ");
        }
        io::stdout().flush()?;

        // Read one line
        let mut line = String::new();
        let n = io::stdin().read_line(&mut line)?;
        if n == 0 {
            // EOF (Ctrl-D). If mid-statement, try to evaluate whatever we have.
            if !acc.trim().is_empty() {
                eval_and_print(&acc);
            }
            println!();
            break;
        }

        let trimmed = line.trim_end();

        // Commands only work at the start of a statement
        if acc.is_empty() {
            match trimmed {
                ":q" | ":quit" | ":exit" => break,
                ":h" | ":help" => {
                    println!("Commands: :q to quit, :help for this, :bench for performance test.");
                    continue;
                }
                ":bench" => {
                    let expression = "1 + 2 * (3 - 4) / -5 + (6 * 7) - 8 / 9 + 10 * (11 + 12) - (13 * 14) / 15 + 16 - 17 * 18 / (19 + 20) - 21 + 22 * 23 / 24 - 25 + 26 * (27 - 28) / 29 + 30";
                    let num_iterations = 1000;

                    let start_time = Instant::now();
                    for _ in 0..num_iterations {
                        evaluate_lines(expression);
                    }
                    let elapsed_time = start_time.elapsed();

                    println!("Benchmarking {}:", expression);
                    println!("  Iterations: {}", num_iterations);
                    println!("  Total time: {:?}", elapsed_time);
                    println!(
                        "  Average time per evaluation: {:?}",
                        elapsed_time / num_iterations
                    );
                    continue;
                }
                cmd if cmd.starts_with(":save")
                    || cmd.starts_with(":w")
                    || cmd.starts_with(":wq") =>
                {
                    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
                    let filename = if parts.len() > 1 && !parts[1].is_empty() {
                        parts[1].trim()
                    } else {
                        "history"
                    };

                    if let Err(e) = save_output(filename, &acc) {
                        error!("Error saving output: {}", e);
                    }
                    acc.clear();
                    if cmd.starts_with(":wq") {
                        break;
                    }
                    continue;
                }
                _ => {}
            }
        }

        // Append the line to the accumulator (keep the newline; your preprocessor handles it)
        acc.push_str(trimmed);
        acc.push('\n');

        // If the visible line (ignoring trailing spaces) ends with a backslash, keep collecting
        let ends_with_backslash = trimmed.trim_end().ends_with('\\');

        if !ends_with_backslash {
            // We’ve got a complete statement (or multiple statements pasted at once).
            eval_and_print(&acc);
            acc.clear();
        }
    }

    Ok(())
}

/// Saves the accumulated REPL input to a file.
///
/// This function handles appending the `.arith` extension if not present
/// and ensures that double extensions like `.arith.arith` are avoided.
///
/// # Arguments
/// * `filename` - The desired name of the file to save to.
/// * `content` - The string content to write to the file.
///
/// # Returns
/// A `std::io::Result<()>` indicating success or an I/O error during file writing.
fn save_output(filename: &str, content: &str) -> io::Result<()> {
    let mut file_path = filename.to_string();

    // Handle double extensions
    if file_path.ends_with(".arith.arith") {
        file_path = file_path.strip_suffix(".arith").unwrap().to_string();
    } else if !file_path.ends_with(".arith") {
        file_path.push_str(".arith");
    }

    let path = Path::new(&file_path);
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    println!("Output saved to {}", file_path);
    Ok(())
}

/// Evaluates the given input string using the `evaluate_lines` orchestrator
/// and prints the results or errors to the console.
///
/// # Arguments
/// * `input` - The string containing one or more logical arithmetic expressions.
fn eval_and_print(input: &str) {
    // orchestrator can accept multiple logical lines; we'll pass the whole chunk.
    let results = evaluate_lines(input);

    // Print each result on its own line in order
    for res in results {
        match res {
            Ok((v, _)) => println!("= {}", fmt_num(v)),
            Err(e) => error!("! {}", e), // assumes EvalError: Display
        }
    }
}

/// Formats a floating-point number (`f64`) for display, removing unnecessary
/// trailing zeros and ensuring integer values are displayed without a decimal point.
///
/// # Arguments
/// * `x` - The `f64` number to format.
///
/// # Returns
/// A `String` representation of the formatted number.
fn fmt_num(x: f64) -> String {
    // Show as integer if it is exactly an integer, else as trimmed float
    if x.fract() == 0.0 && x.is_finite() {
        format!("{}", x as i64)
    } else {
        // 15 sig figs is a decent default without getting noisy
        let s = format!("{:.15}", x);
        // trim trailing zeros and possible trailing dot
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}
