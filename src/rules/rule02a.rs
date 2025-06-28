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

//! # Rule II:A
//!
//! ```text
//!   A. Each line must be kept within 80 columns in order to make sure
//!      the entire line will fit on printouts.  If the line is too long,
//!      then it must be broken up into readable segments.
//!      The indentation of the code on the following lines needs to be
//!      at least 2 spaces.
//!
//!
//!      Example: room_temperature = list_head->left_node->
//!                                           left_node->
//!                                           left_node->
//!                                           left_node->
//!                                           left_node->
//!                                           temperature;
//!
//!      Example: fread(&value, sizeof(double),
//!                     1, special_fp);
//! ```

use indoc::indoc;
use tree_sitter::Range;

use crate::diagnostic::{Diagnostic, SourceRange, Span};
use crate::helpers::line_width;
use crate::{helpers::QueryHelper, rules::api::Rule};

use crate::rules::api::SourceInfo;

use super::api::RuleDescription;

/// Amount that wrapped lines must be indented, in columns.
const WRAPPED_LINE_INDENT_WIDTH: usize = 2;

/// # Rule II:A.
///
/// See module-level documentation for details.
pub struct Rule02a {}

/// Tree-sitter query for Rule II:A.
const QUERY_STR: &str = indoc! { /* query */ r##"
    ; If statement condition
    (if_statement
        condition: (_) @splittable)

    ; Switch statement condition
    (switch_statement
        condition: (_) @splittable)

    ; Case expression
    (case_statement
        value: (_) @splittable)

    ; While loop condition
    (while_statement
        condition: (_) @splittable)

    ; Do-while loop condition
    (do_statement
        condition: (_) @splittable)

    ; For loop parentheses. Here we need separate start and end
    ; captures since a capture can only capture one node.
    (for_statement
        "(" @splittable.begin
        _
        ")" @splittable.end)

    ; Expression statement
    (expression_statement) @splittable

    ; Return statement
    (return_statement) @splittable

    ; Break statement
    (break_statement) @splittable

    ; Continue statement
    (continue_statement) @splittable

    ; Goto statemnt
    (goto_statement) @splittable

    ; Macro definitions
    (preproc_function_def
        "#define" @splittable.begin
        value: (_) @splittable.end)

    ; Variable initialization
    (declaration
        declarator: (init_declarator)) @splittable
"## };

impl Rule for Rule02a {
    fn describe(&self) -> &'static RuleDescription {
        &RuleDescription {
            group_number: 2,
            letter: 'A',
            code: "II:A",
            name: "LineLength",
            description: "lines must be 80 columns wide or less",
        }
    }

    fn check<'a>(
        &self,
        SourceInfo {
            filename,
            tree,
            code,
            lines,
        }: &'a SourceInfo,
    ) -> Vec<Diagnostic<'a>> {
        let mut diagnostics = Vec::new();

        // Check for lines >80 columns long
        for (i, &(line, index)) in lines.iter().enumerate() {
            let width = line_width(line);
            if width > 80 {
                diagnostics.push(
                    self.report("Line length exceeds 80 columns.").with_violation_parts(
                        filename,
                        SourceRange {
                            bytes: (index + 80)..(index + line.len()),
                            start_pos: (i, 80),
                            end_pos: (i, index + line.len()),
                        },
                        "", // FIXME: empty string is ugly
                    ),
                );
            }
        }

        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let splittable_capture_i = helper.expect_index_for_capture("splittable");
        let splittable_begin_capture_i = helper.expect_index_for_capture("splittable.begin");
        let splittable_end_capture_i = helper.expect_index_for_capture("splittable.end");
        helper.for_each_match(|qmatch| {
            // Expect either @splittable or a pair of @splittable.begin and @splittable.end
            assert!(qmatch.captures.len() == 1 || qmatch.captures.len() == 2);

            // Get range from capture
            let range = match qmatch.captures.len() {
                1 => {
                    let node = helper.expect_node_for_capture_index(qmatch, splittable_capture_i);
                    node.range()
                }
                2 => {
                    let start_node =
                        helper.expect_node_for_capture_index(qmatch, splittable_begin_capture_i);
                    let end_node =
                        helper.expect_node_for_capture_index(qmatch, splittable_end_capture_i);
                    Range {
                        start_byte: start_node.start_byte(),
                        end_byte: end_node.end_byte(),
                        start_point: start_node.start_position(),
                        end_point: end_node.end_position(),
                    }
                }
                n => panic!("Expected 1 or 2 captures, got {n}"),
            };

            // If not split across two lines, skip this match
            if range.start_point.row == range.end_point.row {
                return;
            }

            // Check indentation of wrapped lines and construct list of labels
            let mut code_lines = lines.iter().enumerate()
            .skip(range.start_point.row)
            .take(range.end_point.row + 1 - range.start_point.row);
            let (first_line_index, &(first_line, first_line_byte_pos)) = code_lines.next().unwrap();
            let first_line_indent = get_indentation(first_line);
            let first_line_indent_width = line_width(first_line_indent);
            let expected_indent_width = first_line_indent_width + WRAPPED_LINE_INDENT_WIDTH;
            let mut violations = Vec::new();
            for (i, &(this_line, this_line_pos)) in code_lines {
                let this_line_indent = get_indentation(this_line);
                let this_line_indent_width = line_width(this_line_indent);
                if this_line_indent_width < expected_indent_width {
                    violations.push(
                        Span::new(filename, SourceRange { bytes: this_line_pos..(this_line_pos + this_line_indent.len()), start_pos: (i, 0), end_pos: (i, this_line_indent.len()) },
                            format!(
                                "Expected >={expected_indent_width} columns of indentation on continuing line"
                            )),
                    );
                }
            }

            // If no labels, these lines pass the test
            if violations.is_empty() {
                return;
            }

            diagnostics.push(
                Diagnostic::new(self.describe(), format!(
                        "Wrapped expressions/statements must be indented by at least {WRAPPED_LINE_INDENT_WIDTH} spaces",
                    ))
                    .with_violations(violations)
                    .with_reference_parts(
                        filename,
                        SourceRange {
                            bytes: first_line_byte_pos..(first_line_byte_pos + first_line_indent.len()),
                            start_pos: (first_line_index, 0),
                            end_pos: (first_line_index, first_line_indent.len()),
                        },
                        format!(
                            "Found indentation of {first_line_indent_width} columns on initial line",
                        )
                    ),
            );
        });

        diagnostics
    }
}

/// Returns the leading whitespace part of the line
fn get_indentation(line: &str) -> &str {
    &line[0..(line.len() - line.trim_start().len())]
}

#[cfg(test)]
mod tests {
    use std::process::ExitCode;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::{
        helpers::testing::test_captures,
        rules::api::{Rule, SourceInfo},
    };

    use super::{Rule02a, QUERY_STR};

    #[test]
    fn test_rule02a_captures() -> ExitCode {
        let code = indoc! { /* c */ r#"
            int global_var = 10;
            //!? splittable
            int global_var
            //!? splittable
                = 10;

            #define MAX(a, b) (a < b ? b : a)
            //!? splittable.begin
                              //!? splittable.end
            #define MAX(a, b) \
            //!? splittable.begin
              (a < b ? b : a)
              //!? splittable.end
            #define MAX(a, b) \
            //!? splittable.begin
            (a < b ? b : a)
            //!? splittable.end

            int main() {
                int global_var = 10;
                //!? splittable
                int global_var
                //!? splittable
                    = 10;

                x + 2;
                //!? splittable
                x
                //!? splittable
                + 2;

                if (something) abort();
                   //!? splittable
                               //!? splittable
                if (some ||
                   //!? splittable
                    thing) abort();
                           //!? splittable

                while (something) abort();
                      //!? splittable
                                  //!? splittable
                while (some ||
                      //!? splittable
                       thing) abort();
                              //!? splittable

                for (;;) {}
                    //!? splittable.begin
                       //!? splittable.end

                printf("This is %s with %s",
                //!? splittable
                       "a format string",
                       "many lines of arguments");
            }
        "# };
        test_captures(QUERY_STR, code)
    }

    #[test]
    fn test_rule02a_diagnostics() {
        let rule = Rule02a {};

        macro_rules! test {
            ($code:literal, $ndiag:expr, $nlabels_list:expr) => {
                let inner_code = ::indoc::indoc! { $code };
                let mut code = String::new();
                code.push_str("int main() {\n");
                for line in inner_code.lines() {
                    code.push_str("  ");
                    code.push_str(line);
                    code.push('\n');
                }
                code.push_str("}\n");
                dbg!(&code);
                let source = SourceInfo::new("", &code);
                let diagnostics = rule.check(&source);
                assert_eq!($ndiag, diagnostics.len());
                let nlabels_list: &[usize] = &$nlabels_list;
                assert_eq!(
                    nlabels_list,
                    &diagnostics
                        .iter()
                        .map(|diag| diag.violations.len() + diag.references.len())
                        .collect::<Vec<usize>>()
                );
            };
        }

        // Each test takes the code, number of expected diagnostics, and total number of expected
        // spans

        test!("int x = 0;", 0, []);
        test!("int x =\n  0;", 0, []);
        test!("int x =\n0;", 1, [2]);
        test!("for (int i = 0; i < n; i++) {}", 0, []);
        test!("for (int i = 0;\ni < n;\ni++) {}", 1, [3]);
        test!("for (int i = 0;\n  i < n;\n  i++) {}", 0, []);
        test!(
            "
            if (my_condition() == true) {
              data->
                el->other = false;
            }
        ",
            0,
            []
        );
        test!(
            "
            if (my_condition()
                == true) {
              data->
                el->other = false;
            }
        ",
            0,
            []
        );
        test!("#define MAX(a, b) \\\n((a) < (b) ? (a) : (b))", 1, [2]);
        test!("#define MAX(a, b) \\\n  ((a) < (b) ? (a) : (b))", 0, []);
    }
}
