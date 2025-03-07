//! # Rule I:C
//!
//! ```text
//! C. All constants must be all uppercase, and contain at least two
//!   characters.  Constants must be declared using #define.
//!   A constant numeric value assigned must be enclosed in
//!   parenthesis.
//!
//!   String constants need to be placed in quotes but do not
//!   have surrounding parentheses.
//!
//!   Example: #define TEMPERATURE_OF_THE_ROOM (10)
//!
//!   Example: #define FILE_NAME  "Data_File"
//! ```
//!
//! # Implementation notes
//!
//! - Like [Rule I:A][crate::rules::rule1a], it's not possible to check that multi-word identifiers
//!   are separated by underscores.
//!
//! - Currently, values which contain constant numeric expressions with operators will not be
//!   checked for being surrounded with parentheses. For example, `#define ABC 3` gets flagged but
//!   `#define ABC 1 + 2` doesn't. Fixing this will require re-parsing all `preproc_arg` nodes, as
//!   the current [tree-sitter-c][tree_sitter_c] grammar treats them as literal text.

use tree_sitter::{Point, QueryCapture, Tree};

use crate::{rules::api::Rule, helpers::QueryHelper};

/// Tree-sitter query for Rule I:C.
const QUERY_STR: &'static str = r#"
    ; tsquery
    (
        (preproc_def name: (identifier) @constant.name.short)
        (#match? @constant.name.short "^.$")
    )
    (
        (preproc_def name: (identifier) @constant.name.contains_lower)
        (#match? @constant.name.contains_lower "[a-z]")
    )
    (
        (preproc_def value: (preproc_arg) @constant.value.unwrapped_number)
        (#match? @constant.value.unwrapped_number "^[0-9]+$")
    )
"#;

/// # Rule I:C.
///
/// See module-level documentation for details.
pub struct Rule1c {}

impl Rule for Rule1c {
    fn check(&self, filename: &str, tree: &Tree, code: &[u8]) {
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        helper.for_each_capture(|name: &str, capture: QueryCapture| {
            let message: &str = match name {
                "constant.name.short" => "Constant name must contain at least 2 characters",
                "constant.name.contains_lower" => "Constant name must use upper snake case",
                "constant.value.unwrapped_number" => {
                    "Numeric constant value must be wrapped in parentheses"
                }
                _ => unreachable!(),
            };
            let loc: Point = capture.node.start_position();
            println!("{}:{}:{}: {}", filename, loc.row, loc.column, message);
            println!("{}", capture.node.utf8_text(code).unwrap());
        });
    }
}
