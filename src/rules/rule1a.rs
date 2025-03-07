//! # Rule I:A
//!
//! ```text
//!    A. Variable names should be in all lowercase.
//!       If the name is composed of more than one word, then underscores
//!       must be used to separate them.
//!
//!       Example: int temperature = 0;
//!
//!       Example: int room_temperature = 0;
//! ```
//!
//! # Implementation notes
//!
//! This rule checks that all declared identifiers are in lowercase. It cannot check whether
//! underscores are used to separate words because splitting an identifier into words is
//! subjective.

use tree_sitter::{Query, QueryCapture, QueryCursor, StreamingIterator as _, Tree};

use crate::rules::api::Rule;

const QUERY_STR: &'static str = r#"
    ; tsquery
    (
        (_ declarator: (identifier) @name)
        (#match? @name "[A-Z]")
    )
"#;

/// # Rule I:A.
///
/// See module-level documentation for details.
pub struct Rule1a {}

impl Rule for Rule1a {
    fn check(&self, filename: &str, tree: &Tree, code: &[u8]) {
        let query =
            Query::new(&tree_sitter_c::LANGUAGE.into(), QUERY_STR).expect("Failed to create query");
        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), code);
        while let Some((this_match, capture_index)) = captures.next() {
            let capture: QueryCapture = this_match.captures[*capture_index];
            println!(
                "{}:{}:{}: Name \"{}\" must be in lower snake case.",
                filename,
                capture.node.start_position().row,
                capture.node.start_position().column,
                capture
                    .node
                    .utf8_text(code)
                    .expect("Code is not valid UTF-8")
            );
        }
    }
}
