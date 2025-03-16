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

//! # Rule III:A
//!
//! ```text
//!    B. One space must be placed before and after all logical, and
//!       arithmetic operators; except for unary and data reference
//!       operators (i.e. [], ., &, *, ->).
//!
//!       Example: temperature = room_temperature + offset;
//!
//!       Example: temperature = node->data;
//!
//!       Example: if (-temperature == room_temperature)
//!
//!       Example: for (i = 0; i < limit; ++i)
//!
//!       Example: *value = head->data;
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::{Node, Tree};

use crate::{helpers::QueryHelper, rules::api::Rule};

/// Tree-sitter query to capture binary expressions/operators.
const QUERY_STR_BINARY: &str = indoc! {
    /* query */
    r#"
    (binary_expression
        left: _ @prev
        operator: _ @binary-operator
        right: _ @next)
    "#
};

/// Tree-sitter query to capture unary expressions/operators.
const QUERY_STR_UNARY: &str = indoc! {
    /* query */
    r#"
    (unary_expression
        operator: _ @unary-operator
        argument: _ @next)
    (pointer_expression
        operator: _ @unary-operator
        argument: _ @next)
    (pointer_declarator
        "*" @unary-operator
        declarator: _ @next)
    "#
};

/// Tree-sitter query to capture array expressions/operators.
const QUERY_STR_ARRAY: &str = indoc! {
    /* query */
    r#"
    (array_declarator
        declarator: _ @prev
        "[" @array-bracket-left)
    (subscript_expression
        argument: _ @prev
        "[" @array-bracket-left)
    "#
};

/// Tree-sitter query to capture field expressions/operators.
const QUERY_STR_FIELD: &str = indoc! {
    /* query */
    r#"
    (field_expression
        argument: _ @prev
        operator: _ @field-operator
        field: _ @next)
    "#
};

/// # Rule III:B.
///
/// See module-level documentation for details.
pub struct Rule3b {}

impl Rule for Rule3b {
    fn check(&self, tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();

        // Binary expressions
        let helper = QueryHelper::new(QUERY_STR_BINARY, tree, code);
        let prev_capture_i = helper.expect_index_for_capture("prev");
        let op_capture_i = helper.expect_index_for_capture("binary-operator");
        let next_capture_i = helper.expect_index_for_capture("next");
        helper.for_each_match(|qmatch| {
            assert_eq!(3, qmatch.captures.len(), "Expected 3 captures for binary expression");
            let prev = helper.expect_node_for_capture_index(qmatch, prev_capture_i);
            let op = helper.expect_node_for_capture_index(qmatch, op_capture_i);
            let next = helper.expect_node_for_capture_index(qmatch, next_capture_i);
            if let Some(diagnostic) = check_binary_op_spacing(op, prev, next, code) {
                diagnostics.push(diagnostic);
            }
        });

        // Unary expressions
        let helper = QueryHelper::new(QUERY_STR_UNARY, tree, code);
        let op_capture_i = helper.expect_index_for_capture("unary-operator");
        let next_capture_i = helper.expect_index_for_capture("next");
        helper.for_each_match(|qmatch| {
            assert_eq!(2, qmatch.captures.len(), "Expected 2 captures for unary expression");
            let op = helper.expect_node_for_capture_index(qmatch, op_capture_i);
            let next = helper.expect_node_for_capture_index(qmatch, next_capture_i);
            // Nodes must be adjacent
            if op.end_byte() != next.start_byte() {
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:B")
                        .with_message("Expected no space after unary operator")
                        .with_labels(vec![Label::primary((), op.end_byte()..next.start_byte())]),
                );
            }
        });

        // Array expressions/declarations
        let helper = QueryHelper::new(QUERY_STR_ARRAY, tree, code);
        let prev_capture_i = helper.expect_index_for_capture("prev");
        let lbrack_capture_i = helper.expect_index_for_capture("array-bracket-left");
        helper.for_each_match(|qmatch| {
            assert_eq!(
                2,
                qmatch.captures.len(),
                "Expected 2 captures for array expression/declaration"
            );
            let prev = helper.expect_node_for_capture_index(qmatch, prev_capture_i);
            let lbrack = helper.expect_node_for_capture_index(qmatch, lbrack_capture_i);
            // Nodes must be adjacent
            if prev.end_byte() != lbrack.start_byte() {
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:B")
                        .with_message("Expected no space before array subscript")
                        .with_labels(vec![Label::primary(
                            (),
                            prev.end_byte()..lbrack.start_byte(),
                        )]),
                );
            }
        });

        // Field access expressions
        let helper = QueryHelper::new(QUERY_STR_FIELD, tree, code);
        let prev_capture_i = helper.expect_index_for_capture("prev");
        let op_capture_i = helper.expect_index_for_capture("field-operator");
        let next_capture_i = helper.expect_index_for_capture("next");
        helper.for_each_match(|qmatch| {
            assert_eq!(3, qmatch.captures.len(), "Expected 3 captures for field access expression");
            let prev = helper.expect_node_for_capture_index(qmatch, prev_capture_i);
            let op = helper.expect_node_for_capture_index(qmatch, op_capture_i);
            let next = helper.expect_node_for_capture_index(qmatch, next_capture_i);
            if let Some(diagnostic) = check_field_op_spacing(op, prev, next) {
                diagnostics.push(diagnostic);
            }
        });

        diagnostics
    }
}

/// Checks the spacing around a binary operator. Returns a [Diagnostic] if the spacing is
/// incorrect. Otherwise returns [None].
fn check_binary_op_spacing(
    op: Node,
    left: Node,
    right: Node,
    code: &[u8],
) -> Option<Diagnostic<()>> {
    // If the adjacent items are on the same line, check that there's a single space between them.
    // If they're on separate lines, we do nothing, and leave it to Rule II:A to check the
    // indentation.
    let left_bad = left.end_position().row == op.start_position().row
        && !is_single_space_between(left, op, code);
    let right_bad = op.end_position().row == right.start_position().row
        && !is_single_space_between(op, right, code);
    let (message, range) = match (left_bad, right_bad) {
        (true, true) => (
            "Expected a single space on each side of binary operator",
            left.end_byte()..right.start_byte(),
        ),
        (true, false) => {
            ("Expected a single space before binary operator", left.end_byte()..op.end_byte())
        }
        (false, true) => (
            "Expected a single space after binary operator",
            op.start_byte()..right.start_byte(),
        ),
        (false, false) => return None,
    };
    Some(
        Diagnostic::warning()
            .with_code("III:B")
            .with_message(message)
            .with_labels(vec![Label::primary((), range)]),
    )
}

/// Checks the spacing around a field access operator. Returns a [Diagnostic] if the spacing is
/// incorrect. Otherwise returns [None].
fn check_field_op_spacing(op: Node, left: Node, right: Node) -> Option<Diagnostic<()>> {
    // If the adjacent items are on the same line, check that there's a single space between them.
    // If they're on separate lines, we do nothing, and leave it to Rule II:A to check the
    // indentation.
    let left_bad = left.end_byte() != op.start_byte();
    let right_bad = op.end_byte() != right.start_byte();
    let (message, range) = match (left_bad, right_bad) {
        (true, true) => (
            "Expected no space around field access operator",
            left.end_byte()..right.start_byte(),
        ),
        (true, false) => (
            "Expected no space before field access operator",
            left.end_byte()..op.start_byte(),
        ),
        (false, true) => (
            "Expected no space after field access operator",
            op.end_byte()..right.start_byte(),
        ),
        (false, false) => return None,
    };
    Some(
        Diagnostic::warning()
            .with_code("III:B")
            .with_message(message)
            .with_labels(vec![Label::primary((), range)]),
    )
}

/// Returns `true` if there is a single space separating the two nodes, else `false`.
fn is_single_space_between(left: Node, right: Node, code: &[u8]) -> bool {
    // TODO: Support UTF-8, not just bytes
    left.end_byte() + 1 == right.start_byte() && code[left.end_byte()] as char == ' '
}

#[cfg(test)]
mod tests {
    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.
}
