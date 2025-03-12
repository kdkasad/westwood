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

//! # Rule I:D
//!
//! ```text
//!  D. All global variables must be started with prefix "g_".
//!     Declarations/definitions should be at the top of the file.
//!
//!      Example: int g_temperature = 0;
//!
//!     Global variable use should be avoided unless absolutely necessary.
//! ```
//!
//! # Implementation notes
//!
//! This rule requires that all top-level declarations come before the first function in the file.
//! The code standard just says "Declarations/definitions should be at the top of the file," so
//! I interpret that as meaning all declarations/definitions and not just global variable
//! declarations.

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::{QueryCapture, Tree};

use crate::{helpers::QueryHelper, rules::api::Rule};

/// Tree-sitter query for Rule I:D.
const QUERY_STR: &str = indoc! {
    /* query */
    r#"
    (
        (_ declarator: (identifier) @global.no_g_prefix)
        (#not-match? @global.no_g_prefix "^g_")
        (#not-has-ancestor? @global.no_g_prefix function_declarator) ; ignore function declarations
        (#not-has-ancestor? @global.no_g_prefix function_definition) ; ignore local declarations
    )

    (translation_unit (function_definition) @function)
    (translation_unit
        [
            (declaration) @declaration.top_level ; variable or function declaration
            (type_specifier) @declaration.top_level ; struct/enum/etc type declaration
            (type_definition) @declaration.top_level ; typedef
        ]
    )
    "#
};

/// # Rule I:D.
///
/// See module-level documentation for details.
pub struct Rule1d {}

impl Rule for Rule1d {
    fn check(&self, tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let mut first_function_position = None;
        let mut diagnostics = Vec::new();
        helper.for_each_capture(|name: &str, capture: QueryCapture| {
            // For captures that aren't problems, process them as needed and return
            match name {
                "function" => {
                    first_function_position = Some(capture.node.byte_range());
                    return;
                }
                "declaration.top_level" if first_function_position.is_some() => (),
                "declaration.top_level" => return,
                _ => (),
            }
            let diagnostic = match name {
                "global.no_g_prefix" => {
                    let message = "Global variables must be prefixed with `g_'";
                    Diagnostic::warning()
                        .with_code("I:D")
                        .with_message(message)
                        .with_labels(vec![
                            Label::primary((), capture.node.byte_range())
                                .with_message("Variable declared here"),
                            Label::secondary((), capture.node.byte_range()).with_message(format!(
                                "Perhaps you meant `g_{}'",
                                capture
                                    .node
                                    .utf8_text(code)
                                    .expect("Code is not valid UTF-8")
                            )),
                        ])
                }
                "declaration.top_level" => {
                    let message =
                        "All top-level declarations must come before function definitions";
                    Diagnostic::warning()
                        .with_code("I:D")
                        .with_message(message)
                        .with_labels(vec![
                            Label::primary((), capture.node.byte_range())
                                .with_message("Declaration occurs here"),
                            // SAFETY: We will have returned if first_function_position is None.
                            Label::secondary((), first_function_position.as_ref().unwrap().clone())
                                .with_message("First function defined here"),
                        ])
                }
                _ => unreachable!(),
            };
            diagnostics.push(diagnostic);
        });
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::testing::test_captures;

    use indoc::indoc;

    use super::QUERY_STR;

    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    #[test]
    fn rule1d() {
        let input = indoc! { /* c */ r#"
            int an_int;
            //!? declaration.top_level
                //!? global.no_g_prefix
            void *a_g_ptr = NULL;
            //!? declaration.top_level
                  //!? global.no_g_prefix

            struct my_struct function_with_params(int *x, char y);
            //!? declaration.top_level

            typedef struct MyStructure {
            //!? declaration.top_level
                char Character;
            } MyType;
            typedef union MyUnion {
            //!? declaration.top_level
                char Character;
                struct {
                    char *(FuncPtr)(int Arg);
                } AnonStruct;
            } MyType;

            char *foo() {
            //!? function
                return NULL;
            }

            int g_int = 1;
            //!? declaration.top_level
            char g_char_array[10];
            //!? declaration.top_level
            struct {
            //!? declaration.top_level
                int x;
            } another_global;
              //!? global.no_g_prefix
        "#};
        test_captures(QUERY_STR, input);
    }
}
