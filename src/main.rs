use std::io::{self, Write};
use arith::executor::evaluate_lines;

pub fn run_repl() -> io::Result<()> {
    println!("arith REPL — enter expressions. Use '\\' for line-continuation. :q to quit.");

    let mut acc = String::new(); // accumulates current statement (may span lines)

    loop {
        // Primary prompt when empty, secondary when continuing
        if acc.is_empty() {
            print!(">> ");
        } else {
            print!("... ");
        }
        io::stdout().flush()?; // don't be cute, flush the prompt

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
                    println!("Commands: :q to quit, :help for this. Use '\\' to continue lines.");
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

fn eval_and_print(input: &str) {
    // Your orchestrator can accept multiple logical lines; we'll pass the whole chunk.
    let results = evaluate_lines(input);

    // Print each result on its own line in order
    for res in results {
        match res {
            Ok(v) => println!("= {}", fmt_num(v)),
            Err(e) => eprintln!("! {}", e), // assumes EvalError: Display
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

fn main() -> std::io::Result<()> {
    run_repl()
}