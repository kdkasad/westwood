//! Helpers for unit testing.

use std::collections::{HashMap, HashSet};

use tree_sitter::Parser;

use super::QueryHelper;

///
/// Tests a given query's captures on a certain input.
/// Panics if the test fails.
///
/// # Input format
///
/// The input must be a string containing C code interspersed with special comments. These comments
/// indicate expected captures from the query. Each of these "capture comments" must be on its own
/// line, begin with `//!?` and contain one or more expected capture labels. The capture will be
/// expected on the last non-capture-comment line at the column in which the capture comment
/// starts.
///
/// ## Sample input format
///
/// ```c
/// int main() {
/// //!? function
///     return 0;
///     //!? return
///            //!? number
/// }
/// ```
///
/// This will expect a capture named `@function` to start at the first column of the first line.
/// A capture named `@return` will be expected at the start of the `return` keyword. A capture
/// named `@number` will be expected at the `0`.
///
/// # Example
///
/// ```
/// let input = r#"
/// int a;
///     //!? outfunc
/// int b = 0;
///     //!? outfunc
/// int func() {
///     //!? infunc
///     int c;
///         //!? infunc
///     if (a == b) {
///         //!? infunc inif
///              //!? infunc inif
///         int d;
///             //!? infunc inif
///         return d;
///                //!? infunc inif
///     }
/// }
/// "#;
/// let query = r#"
///     ((identifier) @infunc
///         (#has-ancestor? @infunc function_definition))
///     ((identifier) @outfunc
///         (#not-has-ancestor? @outfunc function_definition))
///     ((identifier) @inif
///         (#has-ancestor? @inif if_statement))
/// "#;
/// test_captures(query, input);
/// ```
///
pub fn test_captures(query: &str, input: &str) {
    // Parse input into code and test specs
    let mut code_lines: Vec<&str> = Vec::new();
    let mut test_specs: HashMap<(usize, usize), HashSet<&str>> = HashMap::new();
    for line in input.lines() {
        let trimmed_line = line.trim_start();
        if trimmed_line.starts_with("//!?") {
            let row = code_lines.len() - 1;
            // Get start column of comment
            let col = line.len() - trimmed_line.len();
            // Split into parts and skip "//!?" part
            let labels: HashSet<&str> = trimmed_line.split_whitespace().skip(1).collect();
            assert!(test_specs.insert((row, col), labels).is_none());
        } else {
            code_lines.push(line);
        }
    }
    let code: String = code_lines.join("\n");

    // Run query on code
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
    let tree = parser.parse(&code, None).unwrap();
    let helper = QueryHelper::new(query, &tree, code.as_bytes());
    helper.for_each_capture(|label, capture| {
        let start = capture.node.start_position();
        if let Some(set) = test_specs.get_mut(&(start.row, start.column)) {
            if set.contains(label) {
                set.remove(label);
                if set.is_empty() {
                    test_specs.remove(&(start.row, start.column));
                }
            } else {
                panic!("Unexpected match for @{} at row {} column {}", label, start.row, start.column);
            }
        } else {
            panic!("Unexpected match for @{} at row {} column {}", label, start.row, start.column);
        }
    });
    for ((row, col), set) in test_specs {
        for label in set {
            panic!("Expected @{} at row {} column {}", label, row, col);
        }
    }
}
