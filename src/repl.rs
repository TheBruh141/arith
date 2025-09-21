//! This module implements the Read-Eval-Print Loop (REPL) for the `arith` interpreter.
//!
//! The REPL provides an interactive command-line interface where users can
//! enter arithmetic expressions, and the interpreter will evaluate them and
//! print the results. It supports multi-line input, special commands, and
//! basic error reporting.

use crate::executor::{SimpleExecutor, evaluate_lines};
use log::error;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

pub fn run_repl() -> io::Result<()> {
    println!("arith REPL — enter expressions. Use \\ for line-continuation. :q to quit.");

    let mut acc = String::new();
    let mut executor = SimpleExecutor::new();

    loop {
        if acc.is_empty() {
            print!(">> ");
        } else {
            print!("... ");
        }
        io::stdout().flush()?;

        let mut line = String::new();
        let n = io::stdin().read_line(&mut line)?;
        if n == 0 {
            if !acc.trim().is_empty() {
                eval_and_print(&acc, &mut executor);
            }
            println!();
            break;
        }

        let trimmed = line.trim_end();

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
                        evaluate_lines(expression, &mut executor);
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

        acc.push_str(trimmed);
        acc.push('\n');

        let ends_with_backslash = trimmed.trim_end().ends_with('\\');

        if !ends_with_backslash {
            eval_and_print(&acc, &mut executor);
            acc.clear();
        }
    }

    Ok(())
}

fn save_output(filename: &str, content: &str) -> io::Result<()> {
    let mut file_path = filename.to_string();

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

fn eval_and_print(input: &str, executor: &mut SimpleExecutor) {
    let results = evaluate_lines(input, executor);

    for res in results {
        match res {
            Ok((v, _)) => println!("= {}", fmt_num(v)),
            Err(e) => error!("! {}", e),
        }
    }
}

fn fmt_num(x: f64) -> String {
    if x.fract() == 0.0 && x.is_finite() {
        format!("{}", x as i64)
    } else {
        let s = format!("{:.15}", x);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}
