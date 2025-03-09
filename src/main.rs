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
use tree_sitter::{Parser, Tree};

pub mod helpers;
pub mod rules;

fn usage() {
    let maybe_argv0: Option<String> = std::env::args().nth(0);
    let executable: &str = maybe_argv0.as_deref().unwrap_or("westwood");
    eprintln!("Usage: {} <file>", executable);
}

fn main() -> ExitCode {
    // Get file from args
    let mut args = std::env::args();
    let filename: String = match args.nth(1) {
        Some(arg) => arg,
        None => {
            eprintln!("Error: Expected a filename");
            usage();
            return ExitCode::FAILURE;
        }
    };
    if args.len() != 0 {
        eprintln!("Error: Too many arguments provided");
        usage();
        return ExitCode::FAILURE;
    }

    // Read file
    let code: Vec<u8> = match std::fs::read(&filename) {
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

    // Do checks
    let rules: Vec<Box<dyn Rule>> = crate::rules::get_rules();
    for rule in rules {
        rule.check(&filename, &tree, &code);
    }

    return ExitCode::SUCCESS;
}
