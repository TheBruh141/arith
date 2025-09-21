use crate::executor::{SimpleExecutor, evaluate_lines};
use std::fs::read_to_string;
use std::path::Path;

pub fn run_file_mode(files: Vec<String>) -> std::io::Result<()> {
    for file_path_str in files {
        let path = Path::new(&file_path_str);
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&file_path_str);

        log::info!("Processing file: {}", file_name);
        let content = read_to_string(&file_path_str)?;

        println!("--- Results from {} ---", file_name);

        let mut executor = SimpleExecutor::new();
        let results = evaluate_lines(&content, &mut executor);

        for (idx, result) in results.into_iter().enumerate() {
            match result {
                Ok((val, expr_str)) => println!("{} [{}]: {}", expr_str, idx + 1, val),
                Err(e) => {
                    eprintln!("Error in {}: {}", file_name, e);
                }
            }
        }
        println!();
    }
    Ok(())
}
