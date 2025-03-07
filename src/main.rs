use std::process::ExitCode;

use crate::rules::api::Rule;
use tree_sitter::{Language, Parser, Query, QueryCaptures, QueryCursor, StreamingIterator, Tree};

mod rules;
mod helpers;

fn usage() {
    let maybe_argv0: Option<String> = std::env::args().nth(0);
    let executable: &str = maybe_argv0.as_deref().unwrap_or("westwood");
    eprintln!("Usage: {} <file>", executable);
}

fn find_errors(tree: &Tree, code: &[u8]) -> (u64, u64) {
    let lang: Language = Language::from(tree_sitter_c::LANGUAGE);
    let q_err_src = "(ERROR) @error";
    let q_err = Query::new(&lang, q_err_src).expect("Failed to create query");
    let mut cursor = QueryCursor::new();
    let mut captures: QueryCaptures<_, _> = cursor.captures(&q_err, tree.root_node(), code);
    let mut n_errors: u64 = 0;
    while let Some(_capture) = captures.next() {
        n_errors += 1;
    }
    let q_missing_src = "(MISSING) @missing";
    let q_missing = Query::new(&lang, &q_missing_src).expect("Failed to create query");
    captures = cursor.captures(&q_missing, tree.root_node(), code);
    let mut n_missing = 0;
    while let Some(_capture) = captures.next() {
        n_missing += 1;
    }
    return (n_errors, n_missing);
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
    let (n_errors, n_missing) = find_errors(&tree, &code);
    if n_errors > 0 || n_missing > 0 {
        eprintln!("Found {} syntax errors.", n_errors);
        eprintln!("Ensure your code compiles before running the linter.");
        return ExitCode::FAILURE;
    }

    // Do checks
    let rules: Vec<Box<dyn Rule>> = crate::rules::get_rules();
    for rule in rules {
        rule.check(&filename, &tree, &code);
    }

    return ExitCode::SUCCESS;
}
