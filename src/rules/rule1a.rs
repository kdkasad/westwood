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

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::Tree;

use crate::{helpers::QueryHelper, rules::api::Rule};

const QUERY_STR: &str = indoc! { /* query */ r#"
    (
        [
            (_ declarator: [
                (identifier) @name ; handles variables
                (field_identifier) @name ; handles struct/union fields
                (type_identifier) @name ; handles typedefs
                (parenthesized_declarator (_) @name) ; handles parenthesized names, e.g. function pointers
            ])
            (struct_specifier ; handles struct declarations
                name: (type_identifier) @name
                body: (_))
            (union_specifier ; handles union declarations
                name: (type_identifier) @name
                body: (_))
            (enum_specifier ; handles enum declarations
                name: (type_identifier) @name
                body: (_))
        ]
        (#match? @name "[A-Z]")
    )
"# };

/// # Rule I:A.
///
/// See module-level documentation for details.
pub struct Rule1a {}

impl Rule for Rule1a {
    fn check(&self, tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let mut diagnostics = Vec::new();
        helper.for_each_capture(|_label, capture| {
            let nametype = match capture.node.parent().unwrap().kind() {
                "function_declarator" => "Function",
                "struct_specifier" => "Struct",
                "union_specifier" => "Union",
                "enum_specifier" => "Enum",
                "type_definition" => "Type",
                _ => "Variable",
            };
            let diagnostic = Diagnostic::warning()
                .with_message(format!("{} names must be in lower snake case.", nametype))
                .with_code("I:A")
                .with_labels(vec![
                    Label::primary((), capture.node.byte_range())
                        .with_message("Name contains uppercase character(s)"),
                    Label::secondary((), capture.node.byte_range()).with_message(format!(
                        "Perhaps you meant `{}'",
                        guess_lower_snake_case(
                            capture
                                .node
                                .utf8_text(code)
                                .expect("Code is not valid UTF-8")
                        )
                    )),
                ]);
            diagnostics.push(diagnostic);
        });
        diagnostics
    }
}

/// Attempts to convert a name to lower snake case.
///
/// If the name contains a lowercase character followed by an uppercase one, the two will be
/// converted to lowercase and separated by an underscore.
fn guess_lower_snake_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut last_was_lower = false;
    for (i, c) in name.char_indices() {
        if c.is_uppercase() {
            if i != 0 && last_was_lower {
                result.push('_');
            }
            c.to_lowercase().for_each(|newc| result.push(newc));
            last_was_lower = false;
        } else {
            if c.is_lowercase() {
                last_was_lower = true;
            }
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::helpers::testing::test_captures;
    use pretty_assertions::assert_eq;

    #[test]
    fn guess_lower_snake_case() {
        let tests: Vec<(&str, &str)> = vec![
            ("MYSTR", "mystr"),
            ("myStr", "my_str"),
            ("MY_STR", "my_str"),
            ("thisIsAStruct", "this_is_astruct"),
        ];
        for (input, expected) in tests {
            assert_eq!(expected, super::guess_lower_snake_case(input));
        }
    }

    #[test]
    fn rule1a() {
        let input = indoc! { /* c */ r#"
            int Name;
                //!? name
            int *Name;
                 //!? name
            int *Name[];
                 //!? name
            int Name[];
                //!? name
            typedef struct MyStructure {
                           //!? name
                char Character;
                     //!? name
            } MyType;
              //!? name
            typedef union MyUnion {
                          //!? name
                char Character;
                     //!? name
                struct {
                    char *(FuncPtr)(int Arg);
                           //!? name
                                        //!? name
                } AnonStruct;
                  //!? name
            } MyType;
              //!? name
        "#};
        test_captures(super::QUERY_STR, input);
    }
}
