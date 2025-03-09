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

use indoc::indoc;
use tree_sitter::Tree;

use crate::{helpers::QueryHelper, rules::api::Rule};

const QUERY_STR: &'static str = indoc! { /* query */ r#"
    (
        (_ declarator: (identifier) @name)
        (#match? @name "[A-Z]")
    )
"# };

/// # Rule I:A.
///
/// See module-level documentation for details.
pub struct Rule1a {}

impl Rule for Rule1a {
    fn check(&self, filename: &str, tree: &Tree, code: &[u8]) {
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        helper.for_each_capture(|_label, capture| {
            let start = capture.node.start_position();
            let variable_name = capture
                .node
                .utf8_text(code)
                .expect("Code is not valid UTF-8");
            println!(
                "{}:{}:{}: Name \"{}\" must be in lower snake case.",
                filename, start.row, start.column, variable_name
            );
        });
    }
}
