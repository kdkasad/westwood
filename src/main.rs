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

use std::process::ExitCode;

use crate::rules::api::Rule;
use clap::crate_description;
use clap::Parser as CliArgParser;
use clap_stdin::FileOrStdin;
use codespan_reporting::{
    files::SimpleFile,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use tree_sitter::{Parser, Tree};

pub mod helpers;
pub mod rules;

/// Description printed with `--help` flag
const LONG_ABOUT: &str = concat!("Westwood: ", crate_description!());

#[derive(CliArgParser, Debug)]
#[command(version, about = None, long_about = LONG_ABOUT)]
struct CliOptions {
    /// File to lint, or `-' for standard input
    file: FileOrStdin,
}

fn main() -> ExitCode {
    let cli = CliOptions::parse();

    // Save filename
    let filename = match cli.file.is_file() {
        true => cli.file.filename(),
        false => "(stdin)",
    }
    .to_owned();

    // Read file
    let code: String = match cli.file.contents() {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Error: Cannot read {}: {}", filename, err);
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
    let writer = StandardStream::stdout(ColorChoice::Auto);
    let config = term::Config::default();
    let files = SimpleFile::new(filename, &code);

    // Do checks
    let rules: Vec<Box<dyn Rule>> = crate::rules::get_rules();
    for rule in rules {
        let diagnostics = rule.check(&tree, code.as_bytes());
        for diagnostic in diagnostics {
            term::emit(&mut writer.lock(), &config, &files, &diagnostic)
                .expect("Failed to write diagnostic");
        }
    }

    return ExitCode::SUCCESS;
}
