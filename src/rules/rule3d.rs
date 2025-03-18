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

//! # Rule III:D
//!
//! ```text
//!    D. #define expressions need to be grouped together and need
//!       to be lined up in column 1. They need to have a blank line
//!       above and below. Typically they should go at the top beneath
//!       the includes.
//!
//!       Example: #include "hw1.h"
//!
//!                #define FUNCTION_NAME  "Whatever"
//!                #define UPPER_LIMIT (56)
//!
//!                . . .
//!
//!                /* whatever */
//! ```
//!
//! # Implementation notes
//!
//! Currently, this rule checks
//!  - that top-level `#define` statements come before all function definitions, and
//!  - that all groups of `#define` statements have blank lines before and after, and
//!  - that all `#define` statements within one function are grouped together.
//!
//! It should also (but does not currently) check
//!  - that macros defined in a function are undefined at the end of the function.

use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::{Node, Range as TSRange, Tree};

use crate::{
    helpers::{function_definition_name, QueryHelper, RangeCollapser},
    rules::api::Rule,
};

/// Tree-sitter query for Rule III:D.
const QUERY_STR: &str = indoc! {
    /* query */
    r#"
    ; (preproc_include) @include
    (preproc_def) @define
    (preproc_function_def) @define
    (function_definition
        body: (_) @function.body) @function.definition
    ([(preproc_def) (preproc_function_def)] @define.global
        (#not-has-ancestor? @define.global "function_definition"))

    "#
};

/// # Rule III:D.
///
/// See module-level documentation for details.
pub struct Rule3d {}

impl Rule for Rule3d {
    fn check(&self, tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        // List of function definition bodies
        let mut function_bodies: Vec<Node> = Vec::new();
        // List of #define statements
        let mut definitions: Vec<Node> = Vec::new();
        // List of #define statements outside of functions
        let mut global_definitions: Vec<Node> = Vec::new();
        // Keep track of first function
        let mut first_func: Option<Node> = None;

        let mut diagnostics = Vec::new();

        let helper = QueryHelper::new(QUERY_STR, tree, code);
        helper.for_each_capture(|label, capture| match label {
            "function.body" => function_bodies.push(capture.node),
            "define" => definitions.push(capture.node),
            "function.definition" => {
                if first_func.is_none() {
                    first_func = Some(capture.node);
                }
            }
            "define.global" => global_definitions.push(capture.node),
            _ => unreachable!(),
        });

        // Since QueryCursor::captures() returns captures in order, and that's what
        // QueryHelper::for_each_capture() uses under the hood, the lists should already be
        // sorted.
        debug_assert!(function_bodies.is_sorted_by_key(|func| (func.start_byte(), func.end_byte())));
        debug_assert!(definitions.is_sorted_by_key(|def| (def.start_byte(), def.end_byte())));
        debug_assert!(global_definitions.is_sorted_by_key(|def| (def.start_byte(), def.end_byte())));

        // Check that global #define statements come before function definitions
        let global_define_groups: Vec<TSRange> =
            RangeCollapser::from(global_definitions.into_iter().map(|def| def.range())).collect();
        for group in global_define_groups.iter() {
            if first_func.is_some_and(|func| func.end_byte() < group.start_byte) {
                let print_range =
                    range_without_trailing_eol(group.start_byte..group.end_byte, code);
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:D")
                        .with_message("Global preprocessor definitions must be placed at the top of the file, before all functions")
                        .with_labels(vec![
                            Label::primary((), print_range).with_message("Macro(s) defined here"),
                            // SAFETY: We've already checked that first_func.is_some_and(...).
                            Label::secondary((), first_func.unwrap().byte_range()).with_message("First function defined here")
                        ])
                );
            }
        }

        // Check that global #define statements are grouped together
        if global_define_groups.len() > 1 {
            diagnostics.push(
                Diagnostic::warning()
                    .with_code("III:D")
                    .with_message("All top-level #define statements must be grouped together")
                    .with_labels(
                        global_define_groups
                            .into_iter()
                            .enumerate()
                            .map(|(i, group)| {
                                let range = range_without_trailing_eol(
                                    group.start_byte..group.end_byte,
                                    code,
                                );
                                if i == 0 {
                                    Label::secondary((), range).with_message(
                                        "First group of #define statements found here",
                                    )
                                } else {
                                    Label::primary((), range)
                                        .with_message("More #define statements found here")
                                }
                            })
                            .collect(),
                    ),
            );
        }

        // Get lines of the source
        let lines: Vec<&str> =
            std::str::from_utf8(code).expect("Code is not valid UTF-8").lines().collect();

        // Collapse #define statements into groups
        let define_groups = RangeCollapser::from(definitions.into_iter().map(|def| def.range()));

        // Ensure all #define's in the same function are grouped together
        for function in function_bodies {
            let groups_in_function: Vec<TSRange> = define_groups
                .clone()
                .skip_while(|define| define.start_byte < function.start_byte())
                .take_while(|define| define.end_byte <= function.end_byte())
                .collect();
            if groups_in_function.len() > 1 {
                let function_def =
                    function.parent().expect("Expected function body to have a parent");
                let function_name = function_definition_name(function_def, code);
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:D")
                        .with_message(
                            "All #define statements in each function must be grouped together",
                        )
                        .with_notes(vec![format!("In function `{}()'", function_name)])
                        .with_labels(
                            groups_in_function
                                .into_iter()
                                .enumerate()
                                .map(|(i, define_group)| {
                                    let range = define_group.start_byte..define_group.end_byte;
                                    let print_range = range_without_trailing_eol(range, code);
                                    if i == 0 {
                                        Label::secondary((), print_range).with_message(
                                            "First group of #define statements found here",
                                        )
                                    } else {
                                        Label::primary((), print_range)
                                            .with_message("More #define statements found here")
                                    }
                                })
                                .collect(),
                        ),
                );
            }
        }

        // Check each group of #define statements for blank lines before/after
        for define in define_groups {
            // preproc_def and preproc_function_def nodes contain the trailing (CR)LF as part of
            // the node's range, so we need to figure out whether it's LF or CRLF in order to
            // remove the trailing newline when printing.
            let print_range = range_without_trailing_eol(define.start_byte..define.end_byte, code);

            // For both of the following checks, we consider no line to count as a blank line, i.e.
            // a #define as the first or last line in a file is valid.

            // Check for blank line before.
            // We can't just use lines.get(...).is_none_or(...) because subtracting from
            // 0 will overflow, which causes a panic.
            let has_blank_before =
                define.start_point.row == 0 || lines[define.start_point.row - 1].is_empty();
            if !has_blank_before {
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:D")
                        .with_message("Expected blank line before #define statement(s)")
                        .with_labels(vec![Label::primary((), print_range.clone())]),
                );
            }

            // If the #define does not end at the start of a line, take the next line
            let end_line = define.end_point.row
                + match define.end_point.column {
                    0 => 0,
                    _ => 1,
                };
            let has_blank_after = lines.get(end_line).is_none_or(|line| line.is_empty());
            if !has_blank_after {
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:D")
                        .with_message("Expected blank line after #define statement(s)")
                        .with_labels(vec![Label::primary((), print_range)]),
                );
            }
        }

        diagnostics
    }
}

/// Returns the byte range of a [Node], excluding the trailing end-of-line sequence if it was
/// included in the node's range.
fn range_without_trailing_eol(mut range: Range<usize>, code: &[u8]) -> Range<usize> {
    match &code[(range.end - 2)..range.end] {
        // \r = 0x0d, \n = 0x0a
        [0x0d, 0x0a] => range.end -= 2,
        [_, 0x0a] => range.end -= 1,
        _ => (),
    }
    range
}

#[cfg(test)]
mod tests {
    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    use codespan_reporting::diagnostic::LabelStyle;
    use indoc::indoc;
    use tree_sitter::Parser;

    use crate::rules::api::Rule;

    use super::Rule3d;

    /// Ensures that `#define` statements are being grouped together and not treated separately.
    #[test]
    fn grouping() {
        let code = indoc! {
            /* c */ r#"
            // comment
            #define A
            #define B
            // comment
            "#
        };
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
        let tree = parser.parse(code.as_bytes(), None).unwrap();
        let rule = Rule3d {};
        let diagnostics = rule.check(&tree, code.as_bytes());
        // Expect 2 diagnostics: one for the non-blank line before the first #define and one for
        // the non-blank line after the second #define.
        assert_eq!(2, diagnostics.len());
    }

    /// Ensures that if for some reason the last line in a file is a `#define` statement that does
    /// not contain a trailing newline, it still gets labeled correctly.
    #[test]
    fn no_eol() {
        let code = "// comment\n#define A";
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
        let tree = parser.parse(code.as_bytes(), None).unwrap();
        let rule = Rule3d {};
        let diagnostics = rule.check(&tree, code.as_bytes());
        assert_eq!(1, diagnostics.len());
        assert_eq!(code.lines().last().unwrap(), &code[diagnostics[0].labels[0].range.clone()]);
    }

    /// Ensures that the logic for blank line checking does not fail if there is no line before or
    /// after the given `#define` statement.
    #[test]
    fn file_start_end() {
        let code = "#define A\n";
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
        let tree = parser.parse(code.as_bytes(), None).unwrap();
        let rule = Rule3d {};
        let diagnostics = rule.check(&tree, code.as_bytes());
        assert!(diagnostics.is_empty());
    }

    /// Ensures that when linting a file using CRLF line endings, the CR does not get labeled as
    /// part of the line.
    #[test]
    fn crlf() {
        let code = "/* comment */\r\n#define A 1\r\n";
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
        let tree = parser.parse(code.as_bytes(), None).unwrap();
        let rule = Rule3d {};
        let diagnostics = rule.check(&tree, code.as_bytes());
        // Sanity checks
        assert_eq!(1, diagnostics.len());
        assert_eq!(LabelStyle::Primary, diagnostics[0].labels[0].style);
        // str::lines() excludes the CRLF.
        let expected_line = code.lines().nth(1).unwrap();
        let actual_line = &code[diagnostics[0].labels[0].range.clone()];
        assert_eq!(expected_line, actual_line);
    }
}
