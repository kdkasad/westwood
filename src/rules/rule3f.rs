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

//! # Rule III:F
//!
//! ```text
//!    F. Never place spaces between function names and the parenthesis
//!       preceding the argument list.
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::Tree;

use crate::{helpers::QueryHelper, rules::api::Rule};

/// Tree-sitter query for Rule III:F.
const QUERY_STR: &str = indoc! {
    /* query */
    r#"
    (function_declarator
        declarator: _ @function
        parameters: (parameter_list . "(" @paren))
    (call_expression
        function: _ @function
        arguments: (argument_list . "(" @paren))
    (preproc_function_def
        name: _ @function
        parameters: (preproc_params . "(" @paren))
    "#
};

/// # Rule III:F.
///
/// See module-level documentation for details.
pub struct Rule3f {}

impl Rule for Rule3f {
    fn check(&self, tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let function_capture_i = helper.expect_index_for_capture("function");
        let paren_capture_i = helper.expect_index_for_capture("paren");
        helper.for_each_match(|qmatch| {
            let function = helper.expect_node_for_capture_index(qmatch, function_capture_i);
            let paren = helper.expect_node_for_capture_index(qmatch, paren_capture_i);

            if function.end_byte() != paren.start_byte() {
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:F")
                        .with_message("Expected no space between function and parenthesis")
                        .with_labels(vec![Label::primary(
                            (),
                            function.end_byte()..paren.start_byte(),
                        )]),
                );
            }
        });
        diagnostics
    }
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
    fn rule3f_captures() -> ExitCode {
        let input = indoc! {
            /* c */ r#"
            #define MAX(a, b) (((a) < (b)) ? (b) : (a))
                    //!? function
                       //!? paren

            int main (int, char **);
                //!? function
                     //!? paren

            int main(int argc, char **argv) {
                //!? function
                    //!? paren
                printf("i = %d, j = %d\n", i, j);
                //!? function
                      //!? paren
                printf ("i = %d, j = %d\n", i, j);
                //!? function
                       //!? paren
                printf
                //!? function
                    ("i = %d, j = %d\n", i, j);
                    //!? paren
            }
            "#
        };
        test_captures(QUERY_STR, input)
    }
}
