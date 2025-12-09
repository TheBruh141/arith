use crate::executor::{SimpleExecutor, evaluate_lines};
use log::error;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

pub fn run_repl() -> io::Result<()> {
    println!("arith REPL. Use \\ for line-continuation. :q to quit, :help for help.");

    let mut executor = SimpleExecutor::new();
    let mut buffer = String::new();

    loop {
        print_prompt(&buffer)?;
        io::stdout().flush()?;

        let mut line = String::new();
        let n = io::stdin().read_line(&mut line)?;
        if n == 0 {
            if !buffer.trim().is_empty() {
                eval_and_print(&buffer, &mut executor);
            }
            println!();
            break;
        }

        let trimmed = line.trim_end();

        // handle top-level commands
        if buffer.is_empty() && handle_command(trimmed, &mut executor, &mut buffer)? {
            continue;
        }

        // accumulate multi-line input
        buffer.push_str(trimmed);
        buffer.push('\n');

        if !trimmed.ends_with('\\') {
            eval_and_print(&buffer, &mut executor);
            buffer.clear();
        }
    }

    Ok(())
}

fn print_prompt(buffer: &str) -> io::Result<()> {
    if buffer.is_empty() {
        print!(">> ");
    } else {
        print!("... ");
    }
    Ok(())
}

/// Returns true if the line was a command and handled
fn handle_command(
    line: &str,
    executor: &mut SimpleExecutor,
    buffer: &mut String,
) -> io::Result<bool> {
    match line {
        ":q" | ":quit" | ":exit" => std::process::exit(0),
        ":h" | ":help" => {
            println!(
                "Commands: :q to quit, :help for help, :bench for performance test, :clear to clear buffer, :save <file>"
            );
            return Ok(true);
        }
        ":clear" => {
            buffer.clear();
            print!("\x1B[2J\x1B[1;1H");
            print!(".\\ Arith Repl\n");

            return Ok(true);
        }
        ":bench" => {
            run_benchmark(executor);
            return Ok(true);
        }
        cmd if cmd.starts_with(":save") || cmd.starts_with(":w") || cmd.starts_with(":wq") => {
            let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
            let filename = if parts.len() > 1 && !parts[1].is_empty() {
                parts[1].trim()
            } else {
                "history"
            };

            if let Err(e) = save_output(filename, buffer) {
                error!("Error saving output: {}", e);
            }
            buffer.clear();
            if cmd.starts_with(":wq") {
                std::process::exit(0);
            }
            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}

fn run_benchmark(executor: &mut SimpleExecutor) {
    let expression = "1 + 2 * (3 - 4) / -5 + (6 * 7) - 8 / 9 + 10 * (11 + 12) - (13 * 14) / 15 + 16 - 17 * 18 / (19 + 20) - 21 + 22 * 23 / 24 - 25 + 26 * (27 - 28) / 29 + 30";
    let iterations = 1000;

    let start = Instant::now();
    for _ in 0..iterations {
        evaluate_lines(expression, executor);
    }
    let elapsed = start.elapsed();

    println!("Benchmarking {}:", expression);
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", elapsed);
    println!("  Avg time: {:?}", elapsed / iterations);
}

fn save_output(filename: &str, content: &str) -> io::Result<()> {
    let mut path = filename.to_string();
    if path.ends_with(".arith.arith") {
        path = path.strip_suffix(".arith").unwrap().to_string();
    } else if !path.ends_with(".arith") {
        path.push_str(".arith");
    }

    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    println!("Output saved to {}", path);
    Ok(())
}

fn eval_and_print(input: &str, executor: &mut SimpleExecutor) {
    for res in evaluate_lines(input, executor) {
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
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
