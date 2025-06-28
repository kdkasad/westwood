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

//! # Rule XII:A
//!
//! ```text
//!    A. No more than one variable may be defined on a single line.
//!
//!       DON'T DO THIS:
//!
//!       int side_a, side_b, side_c = 0;
//!
//!
//!       Do it this way:
//!
//!       int side_a = 0;
//!       int side_b = 0;
//!       int side_c = 0;
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::Node;

use crate::{helpers::QueryHelper, rules::api::Rule};

use crate::rules::api::SourceInfo;

use super::api::RuleDescription;

/// # Rule XII:A.
///
/// See module-level documentation for details.
pub struct Rule12a {}

const QUERY_STR: &str = indoc! {
    /* query */
    r#"
    ; Variable declarations inside function bodies
    (
        (declaration) @declaration
        (#has-ancestor? @declaration "function_definition")
    )

    ; Global variable definitions, except those marked extern
    (
        (declaration
            .
            _ @first-child
        ) @declaration
        (#not-has-ancestor? @declaration "function_definition")
        (#not-eq? @first-child "extern")
    )
    "#
};

impl Rule for Rule12a {
    fn describe(&self) -> &'static RuleDescription {
        &RuleDescription {
            group_number: 12,
            letter: 'A',
            code: "XII:A",
            name: "MultipleDefinitions",
            description: "at most one variable may be defined on a single line",
        }
    }

    fn check(&self, SourceInfo { tree, code, .. }: &SourceInfo) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();

        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let declarator_id = tree
            .language()
            .field_id_for_name("declarator")
            .expect("Expected ID for field `declarator'");
        helper.for_each_capture(|label, capture| {
            match label {
                "declaration" => {
                    // First check to see if this is a function declaration, and if so, skip it.
                    if is_function_declaration(capture.node) {
                        return;
                    }

                    // Get number of declarators
                    let mut cursor = capture.node.walk();
                    let n_declarators =
                        capture.node.children_by_field_id(declarator_id, &mut cursor).count();
                    if n_declarators > 1 {
                        let mut declarators =
                            capture.node.children_by_field_id(declarator_id, &mut cursor);
                        // SAFETY: We know the number of declarators is >1
                        let first_declarator = declarators.by_ref().next().unwrap();
                        diagnostics.push(
                            Diagnostic::warning()
                                .with_code("XII:A")
                                .with_message(
                                    "No more than one variable may be defined on a single line.",
                                )
                                .with_label(
                                    Label::secondary((), first_declarator.byte_range())
                                        .with_message("First definition here"),
                                )
                                .with_labels_iter(declarators.map(|declarator| {
                                    Label::primary((), declarator.byte_range())
                                        .with_message("Additional definition here")
                                })),
                        );
                    }
                }

                // Used in the query but we don't need it here
                "first-child" => (),

                _ => unreachable!(),
            }
        });

        diagnostics
    }
}

/// Decide whether this declaration is a function declaration.
///
/// A declaration is determined to be a function declaration if it is possible to reach
/// a `function_declarator` node by repeatedly traversing to the `declarator` field.
fn is_function_declaration(declaration_node: Node) -> bool {
    let mut maybe_node = Some(declaration_node);
    while let Some(current_node) = maybe_node {
        if current_node.kind() == "function_declarator" {
            return true;
        }
        maybe_node = current_node.child_by_field_name("declarator");
    }
    false
}

#[cfg(test)]
mod tests {
    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    use std::process::ExitCode;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use tree_sitter::Parser;

    use crate::helpers::{testing::test_captures, QueryHelper};

    use super::QUERY_STR;

    /// Test [`is_function_declaration()`][super::is_function_declaration].
    #[test]
    fn is_function_declaration() {
        // Here, every other declaration starting with the first must be a function declaration
        let function_declarations = indoc! {
            /* c */ r"
            int main(void);
            char *get_string(void);
            void nested_declaration(void (*inner)(void));
            "
        };
        let non_function_declarations = indoc! {
            /* c */ r"
            int not_a_function;
            char *string;
            "
        };
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();

        // Check positives
        let tree = parser.parse(function_declarations.as_bytes(), None).unwrap();
        let helper = QueryHelper::new("(declaration) @declaration", &tree, function_declarations);
        helper.for_each_capture(|label, capture| {
            assert_eq!("declaration", label);
            println!("matched {}", &function_declarations[capture.node.byte_range()]);
            assert!(super::is_function_declaration(capture.node));
        });

        // Check negatives
        let tree = parser.parse(non_function_declarations.as_bytes(), None).unwrap();
        let helper =
            QueryHelper::new("(declaration) @declaration", &tree, non_function_declarations);
        helper.for_each_capture(|label, capture| {
            assert_eq!("declaration", label);
            println!("matched {}", &non_function_declarations[capture.node.byte_range()]);
            assert!(!super::is_function_declaration(capture.node));
        });
    }

    #[test]
    fn captures() -> ExitCode {
        let input = indoc! {
            /* c */ r"
            int var2, *var2, var3[10];
            //!? declaration first-child

            // The above should not be caught because it's extern.
            extern char **environ;

            // These should be caught but discarded because they're functions
            void func(void);
            //!? declaration first-child
            char *get_string(void);
            //!? declaration first-child
            void another(void (*inner)(void));
            //!? declaration first-child

            // Now check the exact same things inside a function.
            // We don't capture @first-child inside functions.
            int main() {
                int var2, *var2, var3[10];
                //!? declaration

                // The above should be caught because it's not global.
                extern char **environ;
                //!? declaration

                // These should be caught but discarded because they're functions
                void func(void);
                //!? declaration
                char *get_string(void);
                //!? declaration
                void another(void (*inner)(void));
                //!? declaration
            }
            "
        };
        test_captures(QUERY_STR, input)
    }
}
