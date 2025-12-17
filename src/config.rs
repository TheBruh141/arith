use ariadne::{Color, Label, Report, ReportKind};
use clap::Parser;
use std::ops::Range;
use std::path::PathBuf;
use std::{env, fs};

type ConfigErr = Box<Report<'static, (&'static str, Range<usize>)>>;

#[derive(Debug)]
pub struct CompilerOptions {
    pub debug: bool,
    pub print_ast: bool,
    pub print_ast_indent_size: usize,
    pub print_bytecode: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            debug: false,
            print_ast: false,
            print_ast_indent_size: 4,
            print_bytecode: false,
        }
    }
}

impl CompilerOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn print_ast(mut self, value: bool, size: usize) -> Self {
        self.print_ast = value;
        self.print_ast_indent_size = size;
        self
    }
    pub fn debug_mode() -> Self {
        Self {
            debug: true,
            print_ast: true,
            print_ast_indent_size: 4,
            print_bytecode: true,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version = "0.0.2")]
pub struct Args {
    #[arg(required = true)]
    pub files: Vec<PathBuf>,

    #[arg(
        short,
        long,
        value_parser = clap::value_parser!(bool),
        num_args = 0..=1,
        default_missing_value = "true"
    )]
    pub debug: Option<bool>,

    #[arg(
        long,
        value_parser = clap::value_parser!(bool),
        num_args = 0..=1,
        default_missing_value = "true"
    )]
    pub print_ast: Option<bool>,

    #[arg(long)]
    pub print_ast_indent_size: Option<usize>,

    #[arg(
        long,
        value_parser = clap::value_parser!(bool),
        num_args = 0..=1,
        default_missing_value = "true"
    )]
    pub print_bytecode: Option<bool>,
}

// Holds the reconstructed command line and the spans of specific arguments.
pub struct CliContext {
    // The full command line string (e.g., "arith file1.txt file2.txt")
    pub full_command: String,
    // Maps the index of the file in `args.files` to its span in `full_command`
    arg_spans: Vec<Range<usize>>,
}

impl CliContext {
    pub fn new(parsed_files: &[PathBuf]) -> Self {
        let mut full_command = String::new();
        let mut arg_spans = Vec::new();

        // We iterate over the raw environment arguments to reconstruct the string
        // and find the offsets of the files we care about.
        let raw_args: Vec<String> = env::args().collect();

        for (i, arg) in raw_args.iter().enumerate() {
            let start = full_command.len();
            full_command.push_str(arg);
            let end = full_command.len();

            // Check if this raw argument matches one of our parsed files
            // (Simple matching logic; robust logic handles flags/ordering)
            if parsed_files.iter().any(|p| p.to_string_lossy() == *arg) {
                arg_spans.push(start..end);
            }

            // Add a space for visual reconstruction
            if i < raw_args.len() - 1 {
                full_command.push(' ');
            }
        }

        Self {
            full_command,
            arg_spans,
        }
    }
}

impl Args {
    /// Convert parsed CLI args into a CompilerOptions instance.
    /// Behavior:
    /// 1. Start with default options.
    /// 2. If debug == Some(true) apply debug_mode().
    /// 3. Let explicit flags override whatever debug did.
    pub fn into_compiler_options(self) -> CompilerOptions {
        let mut opts = CompilerOptions::default();

        // Apply debug if explicitly true
        if self.debug == Some(true) {
            opts = CompilerOptions::debug_mode();
        } else if self.debug == Some(false) {
            // explicit false: nothing to enable (keeps defaults), but explicit false
            // is different from None; we don't need special handling here besides
            // letting other explicit flags override.
        }

        // If user explicitly set print_bytecode, override
        if let Some(pb) = self.print_bytecode {
            opts.print_bytecode = pb;
        }

        // If user explicitly set print_ast, override
        if let Some(pa) = self.print_ast {
            opts.print_ast = pa;
        }

        // Indent size: only apply if print_ast was not explicitly disabled.
        // That covers cases:
        //  - debug enabled & user passed indent -> apply
        //  - user passed --print-ast true and indent -> apply
        //  - user passed --print-ast false and indent -> ignore (warn)
        if let Some(size) = self.print_ast_indent_size {
            if self.print_ast == Some(false) {
                // user explicitly disabled print_ast but provided an indent size:
                // ignore the size to avoid confusing state.
                eprintln!(
                    "warning: --print-ast-indent-size provided but --print-ast was explicitly disabled; ignoring indent size"
                );
            } else {
                // Either print_ast was Some(true) or None (maybe debug enabled).
                opts.print_ast_indent_size = size;
            }
        }

        opts
    }
    pub fn check(&self, ctx: &CliContext) -> Result<(), ConfigErr> {
        self.check_files_exist(ctx)?;
        self.check_file_extensions(ctx)?;
        self.check_nonempty_files(ctx)?;
        self.check_no_nul_bytes(ctx)?;
        Ok(())
    }

    fn check_files_exist(&self, ctx: &CliContext) -> Result<(), ConfigErr> {
        for (i, file) in self.files.iter().enumerate() {
            if !file.exists() {
                // Get the span from our context (defaults to 0..0 if logic missed it)
                let span = ctx.arg_spans.get(i).cloned().unwrap_or(0..0);

                return Err(Box::new(
                    Report::build(ReportKind::Error, ("<command line>", span.clone()))
                        .with_message("File not found")
                        .with_label(
                            Label::new(("<command line>", span))
                                .with_message("This file path provided in arguments does not exist")
                                .with_color(Color::Red),
                        )
                        .with_note("Please check the path and try again.")
                        .finish(),
                ));
            }
        }
        Ok(())
    }

    fn check_file_extensions(&self, ctx: &CliContext) -> Result<(), ConfigErr> {
        for (i, file) in self.files.iter().enumerate() {
            let valid_ext = file
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("arith"));

            if !valid_ext {
                let span = ctx.arg_spans.get(i).cloned().unwrap_or(0..0);
                return Err(Box::new(
                    Report::build(ReportKind::Error, ("<command line>", span.clone()))
                        .with_message("Invalid file extension")
                        .with_label(
                            Label::new(("<command line>", span))
                                .with_message("This file must end in .arith")
                                .with_color(Color::Yellow),
                        )
                        .finish(),
                ));
            }
        }
        Ok(())
    }

    fn check_nonempty_files(&self, ctx: &CliContext) -> Result<(), ConfigErr> {
        for (i, file) in self.files.iter().enumerate() {
            // We read to check content, but error points to CLI argument
            let content = fs::read_to_string(file).unwrap_or_default();

            if content.trim().is_empty() {
                let span = ctx.arg_spans.get(i).cloned().unwrap_or(0..0);
                return Err(Box::new(
                    Report::build(ReportKind::Error, ("<command line>", span.clone()))
                        .with_message("Empty source file")
                        .with_label(
                            Label::new(("<command line>", span))
                                .with_message("This file is empty")
                                .with_color(Color::Yellow),
                        )
                        .finish(),
                ));
            }
        }
        Ok(())
    }

    fn check_no_nul_bytes(&self, ctx: &CliContext) -> Result<(), ConfigErr> {
        for (i, file) in self.files.iter().enumerate() {
            let data = fs::read(file).unwrap_or_default();

            if data.contains(&0) {
                let span = ctx.arg_spans.get(i).cloned().unwrap_or(0..0);
                return Err(Box::new(
                    Report::build(ReportKind::Error, ("<command line>", span.clone()))
                        .with_message("Binary file detected")
                        .with_label(
                            Label::new(("<command line>", span))
                                .with_message("This file contains binary data (NUL bytes)")
                                .with_color(Color::Red),
                        )
                        .finish(),
                ));
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod flag_tests {
    use super::*;
    use clap::Parser;
    use std::ffi::OsString;

    // Helper: convenience to create Args from a slice of &str
    fn args_from<I, S>(slice: I) -> Args
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr> + Clone,
        OsString: From<S>,
    {
        Args::try_parse_from(slice).expect("failed to parse args")
    }

    #[test]
    fn default_opts_when_no_flags() {
        let args = args_from(["prog", "file.arith"]);
        let opts = args.into_compiler_options();
        assert!(!opts.debug);
        assert!(!opts.print_ast);
        assert!(!opts.print_bytecode);
        assert_eq!(
            opts.print_ast_indent_size,
            CompilerOptions::default().print_ast_indent_size
        );
    }

    #[test]
    fn debug_enables_prints() {
        let args = args_from(["prog", "--debug=true", "file.arith"]);
        let opts = args.into_compiler_options();
        assert!(
            opts.debug,
            "debug should be enabled when --debug is provided"
        );
        assert!(opts.print_ast, "debug should enable print_ast by default");
        assert!(
            opts.print_bytecode,
            "debug should enable print_bytecode by default"
        );
    }

    #[test]
    fn debug_false_and_explicit_print_ast_false() {
        // --debug false explicitly disables debug; explicit print_ast false overrides anything
        let args = args_from([
            "prog",
            "--debug",
            "false",
            "--print-ast",
            "false",
            "file.arith",
        ]);
        let opts = args.into_compiler_options();
        assert!(!opts.debug);
        assert!(!opts.print_ast);
        assert!(
            !opts.print_bytecode,
            "print_bytecode should remain default/disabled"
        );
    }

    #[test]
    fn debug_then_override_print_ast_false() {
        // --debug (true) sets debug & enables prints, but --print-ast false should override print_ast
        let args = args_from(["prog", "--debug", "--print-ast", "false", "file.arith"]);
        let opts = args.into_compiler_options();
        assert!(opts.debug);
        assert!(
            !opts.print_ast,
            "explicit --print-ast false must override debug's enabling"
        );
        assert!(
            opts.print_bytecode,
            "print_bytecode should remain enabled by debug unless explicitly overridden"
        );
    }

    #[test]
    fn indent_size_applied_when_print_ast_enabled() {
        // debug enables print_ast; explicit indent should be applied
        let args = args_from([
            "prog",
            "--debug",
            "--print-ast-indent-size",
            "8",
            "file.arith",
        ]);
        let opts = args.into_compiler_options();
        assert_eq!(opts.print_ast_indent_size, 8);
    }

    #[test]
    fn indent_ignored_when_print_ast_false() {
        // explicit disabling of print_ast should cause indent to be ignored (and warning printed)
        let args = args_from([
            "prog",
            "--print-ast",
            "false",
            "--print-ast-indent-size",
            "8",
            "file.arith",
        ]);
        let opts = args.into_compiler_options();
        // indent stays at default because print_ast was explicitly disabled
        assert_eq!(
            opts.print_ast_indent_size,
            CompilerOptions::default().print_ast_indent_size
        );
    }
}
