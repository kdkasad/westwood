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
//! - Like [Rule I:A][crate::rules::rule01a], it's not possible to check that multi-word identifiers
//!   are separated by underscores.
//!
//! - Currently, values which contain constant numeric expressions with operators will not be
//!   checked for being surrounded with parentheses. For example, `#define ABC 3` gets flagged but
//!   `#define ABC 1 + 2` doesn't. Fixing this will require re-parsing all `preproc_arg` nodes, as
//!   the current [tree-sitter-c][tree_sitter_c] grammar treats them as literal text.

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::QueryCapture;

use crate::{helpers::QueryHelper, rules::api::Rule};

use crate::rules::api::SourceInfo;

/// Tree-sitter query for Rule I:C.
const QUERY_STR: &str = indoc! { /* query */ r#"
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
"# };

/// # Rule I:C.
///
/// See module-level documentation for details.
pub struct Rule01c {}

impl Rule for Rule01c {
    fn check(&self, SourceInfo { tree, code, .. }: &SourceInfo) -> Vec<Diagnostic<()>> {
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let mut diagnostics = Vec::new();
        helper.for_each_capture(|name: &str, capture: QueryCapture| {
            let node_text = &code[capture.node.byte_range()];
            let (message, label, fix) = match name {
                "constant.name.short" => (
                    "Constant name must contain at least 2 characters",
                    "Constant defined here",
                    None,
                ),
                "constant.name.contains_lower" => (
                    "Constant name must use upper snake case",
                    "Constant defined here",
                    Some(node_text.to_uppercase()),
                ),
                "constant.value.unwrapped_number" => (
                    "Numeric constant value must be wrapped in parentheses",
                    "Value defined here",
                    Some(format!("({node_text})")),
                ),
                _ => unreachable!(),
            };
            let mut diagnostic = Diagnostic::warning()
                .with_code("I:C")
                .with_message(message)
                .with_label(Label::primary((), capture.node.byte_range()).with_message(label));
            if let Some(fix) = fix {
                diagnostic.labels.push(
                    Label::secondary((), capture.node.byte_range())
                        .with_message(format!("Perhaps you meant `{fix}'")),
                );
            }
            diagnostics.push(diagnostic);
        });
        diagnostics
    }
}
