use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;
use log::error;
use crate::executor::evaluate_lines;

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
                    println!("  Average time per evaluation: {:?}", elapsed_time / num_iterations);
                    continue;
                }
                cmd if cmd.starts_with(":save") || cmd.starts_with(":w") || cmd.starts_with(":wq") => {
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

fn eval_and_print(input: &str) {
    // orchestrator can accept multiple logical lines; we'll pass the whole chunk.
    let results = evaluate_lines(input);

    // Print each result on its own line in order
    for res in results {
        match res {
            Ok(v) => println!("= {}", fmt_num(v)),
            Err(e) => error!("! {}", e), // assumes EvalError: Display
        }
    }
}

/// Format f64 without silly trailing zeros.
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
