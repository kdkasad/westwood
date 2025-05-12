// Copyright (C) 2025 Kian Kasad <kian@kasad.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::stdout;
use std::io::IsTerminal;
use std::process::ExitCode;

use crate::rules::api::Rule;
use clap::crate_description;
use clap::Parser as CliArgParser;
use clap::ValueEnum;
use clap_stdin::FileOrStdin;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::diagnostic::LabelStyle;
use codespan_reporting::diagnostic::Severity;
use codespan_reporting::files::Files;
use codespan_reporting::{
    files::SimpleFile,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use crashlog::cargo_metadata;
use tree_sitter::{Parser, Tree};

pub mod helpers;
pub mod rules;

/// Description printed with `--help` flag
const LONG_ABOUT: &str = concat!("Westwood: ", crate_description!());

#[derive(CliArgParser, Debug)]
#[command(version, about = None, long_about = LONG_ABOUT)]
struct CliOptions {
    #[arg(help = "File to lint, or `-' for standard input")]
    file: FileOrStdin,

    #[arg(value_enum, short, long, default_value_t = OutputFormat::Pretty)]
    format: OutputFormat,

    #[arg(value_enum, long, default_value_t = ColorMode::Auto)]
    color: ColorMode,
}

/// Format in which to print diagnostics
#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    /// Pretty human-readable output
    Pretty,

    /// Machine-parseable output
    Machine,
}

/// When to print colored output
#[derive(Copy, Clone, Debug, ValueEnum)]
enum ColorMode {
    /// Never print colored output
    Never,

    /// Print colored output when stdout is a terminal
    Auto,

    /// Always print colored output
    Always,
}

/// Lets us convert from our own [`ColorMode`] type into the [`ColorChoice`] type used by
/// [`codespan_reporting`]. This is also where we check if stdout is a terminal, since
/// [`codespan_reporting`] doesn't do that for us.
impl From<ColorMode> for ColorChoice {
    fn from(val: ColorMode) -> Self {
        match val {
            ColorMode::Never => ColorChoice::Never,
            ColorMode::Auto if !stdout().is_terminal() => ColorChoice::Never,
            ColorMode::Auto => ColorChoice::Auto,
            ColorMode::Always => ColorChoice::Always,
        }
    }
}

fn main() -> ExitCode {
    // Set custom panic handler for release mode
    if !cfg!(debug_assertions) && std::env::var_os("RUST_BACKTRACE").is_none() {
        let mut metadata = cargo_metadata!();
        let mut chars = metadata.package.chars();
        metadata.package = chars
            .next()
            .expect("Expected non-empty package name")
            .to_uppercase()
            .chain(chars)
            .collect::<String>()
            .into();
        crashlog::setup(metadata);
    }

    let cli = CliOptions::parse();

    // Save filename
    let filename = if cli.file.is_file() {
        cli.file.filename()
    } else {
        "(stdin)"
    }
    .to_owned();

    // Read file
    let code: String = match cli.file.contents() {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Error: Cannot read {filename}: {err}");
            return ExitCode::FAILURE;
        }
    };

    // Create parser
    let mut parser: Parser = Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .expect("Error loading C parser grammar");

    // Parse code
    let tree: Tree = parser.parse(&code, None).expect("Failed to parse code");

    // Check for syntax errors
    if tree.root_node().has_error() {
        eprintln!("Found syntax error(s) in your code.");
        eprintln!("Ensure your code compiles before running the linter.");
        eprintln!("To prevent false positives, the linter will not check code with syntax errors.");
        return ExitCode::FAILURE;
    }

    // Create diagnostic writer & file source
    let writer = StandardStream::stdout(cli.color.into());
    // TODO: Detect color (and maybe box drawing) support
    let config = term::Config {
        tab_width: 8,
        ..Default::default()
    };
    let files = SimpleFile::new(filename, &code);

    // Do checks
    let rules: Vec<Box<dyn Rule>> = crate::rules::get_rules();
    for rule in rules {
        let diagnostics = rule.check(&tree, code.as_bytes());
        for diagnostic in diagnostics {
            match cli.format {
                OutputFormat::Pretty => {
                    term::emit(&mut writer.lock(), &config, &files, &diagnostic)
                        .expect("Failed to write diagnostic");
                }
                OutputFormat::Machine => print_machine_parseable(&files, &diagnostic),
            }
        }
    }

    ExitCode::SUCCESS
}

/// Prints a [`Diagnostic`] in a machine-parseable format.
///
/// # Format
///
/// The format is consistent and self-explanatory, so an example should suffice to describe the format:
/// ```text
/// WARNING: [I:D] All top-level declarations must come before function definitions
///          at hw8_main.c from line 212 column 1 to line 217 column 2
/// ```
///
/// Currently, this format does not print any labels other than the primary one, and discards all
/// label messages, keeping only the message of the diagnostic itself.
///
/// # Panics
///
/// This function requires that the given diagnostic has at least one [`Label`] with a style
/// of [`Primary`][2]. If this is not the case, it will panic.
///
/// This function also panics if the primary label of the given diagnostic has a file ID which is
/// not in the given [`Files`] database.
///
/// [2]: codespan_reporting::diagnostic::LabelStyle::Primary
fn print_machine_parseable<'files, F>(files: &'files F, diagnostic: &Diagnostic<F::FileId>)
where
    F: Files<'files, Name: AsRef<str>>,
{
    let primary_label: &Label<_> = diagnostic
        .labels
        .iter()
        .find(|label| label.style == LabelStyle::Primary)
        .expect("Diagnostic has no primary label");
    let filename = files
        .name(primary_label.file_id)
        .expect("Expected to find a file with the given ID");
    let byte_range = &primary_label.range;
    let start = files.location(primary_label.file_id, byte_range.start).unwrap();
    let end = files.location(primary_label.file_id, byte_range.end).unwrap();
    let severity = match diagnostic.severity {
        Severity::Bug => "BUG",
        Severity::Error => "ERROR",
        Severity::Warning => "WARNING",
        Severity::Note => "NOTE",
        Severity::Help => "HELP",
    };
    print!("{severity}: ");
    if let Some(code) = diagnostic.code.as_ref() {
        print!("[{code}] ");
    }
    println!("{}", diagnostic.message);
    println!(
        "{:indent$}at {} from line {} column {} to line {} column {}",
        "",
        filename,
        start.line_number,
        start.column_number,
        end.line_number,
        end.column_number,
        indent = severity.len() + 2,
    );
}
