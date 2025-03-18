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

//! # Rule III:C
//!
//! ```text
//!    C. One space must be placed after internal semi-colons and commas.
//!
//!       Example: for (i = 0; i < limit; ++i)
//!
//!       Example: printf("%f %f %f\n", temperature, volume, area);
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::{Node, Tree};

use crate::{helpers::QueryHelper, rules::api::Rule};

/// Tree-sitter query for Rule III:C.
const QUERY_STR: &str = indoc! {
    /* query */
    r#"
    ; The grammar for for_statement is split into two cases
    (for_statement
        initializer: (declaration ";" @delim .)
        .
        _ @next)
    (for_statement ";" @delim . _ @next)

    ; Struct declaration body
    (field_declaration_list
        (field_declaration ";" @delim .)
        .
        _ @next)

    (argument_list "," @delim . _ @next) ; Function/macro call & attribute arguments
    (parameter_list "," @delim . _ @next) ; Function declaration parameters
    (comma_expression "," @delim . _ @next) ; Comma expressions
    (initializer_list "," @delim . _ @next) ; Initializer lists
    (enumerator_list "," @delim . _ @next) ; Enum lists
    (preproc_params "," @delim . _ @next) ; Macro parameters
    (declaration "," @delim . _ @next) ; Comma-separated declarations
    (type_definition "," @delim . _ @next) ; Comma-separated typedefs
    (attribute_declaration "," @delim . _ @next) ; Attribute lists
    "#
};

/// # Rule III:C.
///
/// See module-level documentation for details.
pub struct Rule3c {}

impl Rule for Rule3c {
    fn check(&self, tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let delim_capture_i = helper.expect_index_for_capture("delim");
        let next_capture_i = helper.expect_index_for_capture("next");
        helper.for_each_match(|qmatch| {
            let delim = helper.expect_node_for_capture_index(qmatch, delim_capture_i);
            let next = helper.expect_node_for_capture_index(qmatch, next_capture_i);

            // Skip if on different lines
            if delim.end_position().row != next.start_position().row {
                return;
            }

            if !is_single_space_between(delim, next, code) {
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:C")
                        .with_message("Expected one space after internal commas and semicolons")
                        .with_labels(vec![Label::primary(
                            (),
                            delim.start_byte()..next.start_byte(),
                        )]),
                );
            }
        });
        diagnostics
    }
}

/// Returns `true` if the two nodes are separated by a single space and `false` otherwise.
fn is_single_space_between(left: Node, right: Node, code: &[u8]) -> bool {
    // TODO: Support UTF-8 and not just bytes
    (left.end_byte() + 1) == right.start_byte() && (code[left.end_byte()] as char) == ' '
}

#[cfg(test)]
mod tests {
    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    use std::process::ExitCode;

    use indoc::indoc;

    use crate::helpers::testing::test_captures;

    use super::QUERY_STR;

    #[test]
    fn rule3c_captures() -> ExitCode {
        let input = indoc! {
            /* c */ r#"
            #define MAX(a, b) (((a) < (b)) ? (b) : (a))
                         //!? delim
                           //!? next

            struct some_struct {
                char first;
                          //!? delim
                char second;
                           //!? delim
                //!? next
            };
            //!? next

            enum weekday {
                SUNDAY, MONDAY,
                      //!? delim
                        //!? next
                              //!? delim
                TUESDAY, WEDNESDAY,
                //!? next
                       //!? delim
                         //!? next
                                  //!? delim
                THURSDAY,
                //!? next
                        //!? delim
                FRIDAY,
                //!? next
                      //!? delim
                SATURDAY,
                //!? next
                        //!? delim
            };
            //!? next

            typedef int i32, integer;
                           //!? delim
                             //!? next

            int main(int argc, char **argv) {
                             //!? delim
                               //!? next
                int i;
                for (i = 0; i < 10; i++) {
                          //!? delim
                            //!? next
                                  //!? delim
                                    //!? next
                    for (int j = i + 1; j < 10; j++) {
                                      //!? delim
                                        //!? next
                                              //!? delim
                                                //!? next
                        printf("i = %d, j = %d\n", i, j);
                                                 //!? delim
                                                   //!? next
                                                    //!? delim
                                                      //!? next
                    }
                }
                printf("%d\n",
                             //!? delim
                       MAX(1,
                       //!? next
                            //!? delim
                           2));
                           //!? next
                struct some_struct foo = { 'a', 'b' };
                                              //!? delim
                                                //!? next
                __attribute__ (( unused, unused ))
                                       //!? delim
                                         //!? next
                struct some_struct foo = { .first = 'a', .second = 'b', };
                                                       //!? delim
                                                         //!? next
                                                                      //!? delim
                                                                        //!? next
                int a, b;
                     //!? delim
                       //!? next
            }
            "#
        };
        test_captures(QUERY_STR, input)
    }
}
