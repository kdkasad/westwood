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

//! # Rule II:B
//!
//! ```text
//!   B. Each function should be kept small for modularity purpose.
//!      The suggested size is less than two pages.
//!      Exception can be made, if the logic of the function requires its
//!      size to be longer than two pages. Common sense needs to be followed.
//!
//!      Example: If a function contains more than two pages of printf
//!               or switch statements, then it would be illogical to break
//!               the function into smaller functions.
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::{Node, QueryCapture, Tree};

use crate::{helpers::QueryHelper, rules::api::Rule};

/// Number of lines per page
const PAGE_SIZE: usize = 61;
/// Maximum number of pages a function definition may span
const MAX_PAGES_PER_FUNCTION: usize = 2;

/// Tree-sitter query for Rule I:D.
const QUERY_STR: &str = indoc! {
    /* query */
    r#"
    (function_definition) @function
    "#
};

/// # Rule II:B.
///
/// See module-level documentation for details.
pub struct Rule2b {}

impl Rule for Rule2b {
    fn check(&self, tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let mut diagnostics = Vec::new();
        helper.for_each_capture(|label: &str, capture: QueryCapture| match label {
            "function" => {
                let start = capture.node.start_position();
                let end = capture.node.end_position();
                let length = end.row - start.row + 1;
                if length > MAX_PAGES_PER_FUNCTION * PAGE_SIZE {
                    let message = format!(
                        "Functions must fit on {} pages, i.e. be no longer than {} lines",
                        MAX_PAGES_PER_FUNCTION,
                        MAX_PAGES_PER_FUNCTION * PAGE_SIZE
                    );
                    let diagnostic = Diagnostic::warning()
                        .with_code("II:B")
                        .with_message(message)
                        .with_labels(vec![Label::primary((), capture.node.byte_range())
                            .with_message(format!(
                                "Function `{}()' is {} lines long",
                                function_definition_name(capture.node, code),
                                length
                            ))]);
                    diagnostics.push(diagnostic);
                }
            }
            _ => unreachable!(),
        });
        diagnostics
    }
}

/// Returns the name of a function defined by a `function_definition` node.
///
/// # Panics
///
/// This function panics if:
/// - the given `node`'s [kind][Node::kind()] is not `function_definition`;
/// - the given `node` does not have an `identifier` child reachable by repeatedly traversing to
///   the node named by the `declarator` field;
/// - the node's text is not valid UTF-8
///
fn function_definition_name<'code>(node: Node, code: &'code [u8]) -> &'code str {
    assert_eq!(
        "function_definition",
        node.kind(),
        "Expected node to have kind `function_definition'"
    );

    let mut node = node;
    while node.kind() != "identifier" {
        node = node
            .child_by_field_name("declarator")
            .expect("Expected node to have a `declarator' field");
    }
    node.utf8_text(code).expect("Code is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use crate::{helpers::QueryHelper, rules::{
        api::Rule,
        rule2b::{MAX_PAGES_PER_FUNCTION, PAGE_SIZE},
    }};

    use codespan_reporting::diagnostic::{Diagnostic, Label};
    use pretty_assertions::assert_eq;
    use tree_sitter::Parser;

    use super::{Rule2b, QUERY_STR};

    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    #[test]
    fn rule2b() {
        // Generate long function
        let mut code = String::new();
        code.push_str("int main() {\n");
        for _ in 0..(PAGE_SIZE * MAX_PAGES_PER_FUNCTION) {
            code.push_str("  (void) 0;\n");
        }
        code.push_str("}\n");

        // Test for diagnostic
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code.as_bytes(), None).unwrap();
        let rule2b = Rule2b {};
        assert_eq!(
            rule2b.check(&tree, code.as_bytes()),
            vec![Diagnostic::warning()
                .with_code("II:B")
                .with_message(format!(
                    "Functions must fit on {} pages, i.e. be no longer than {} lines",
                    MAX_PAGES_PER_FUNCTION,
                    PAGE_SIZE * MAX_PAGES_PER_FUNCTION
                ))
                .with_labels(vec![Label::primary((), 0..(code.len() - 1)).with_message(
                    format!(
                        "Function `main()' is {} lines long",
                        2 + MAX_PAGES_PER_FUNCTION * PAGE_SIZE
                    )
                )])]
        );
    }

    #[test]
    fn function_definition_name() {
        // List of tuples of the form (code, function name)
        let tests = [
            ("int main() {}", "main"),
            ("void **(*ptrptrptr)(char a[])", "ptrptrptr"),
            ("char *strcpy(char *dst, const char *src)", "strcpy"),
            ("char *strdup(const char *src)", "strdup"),
            ("void free(void *ptr)", "free"),
        ];
        for (code, expected_name) in tests {
            let mut parser = Parser::new();
            parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
            let tree = parser.parse(code.as_bytes(), None).unwrap();
            let helper = QueryHelper::new(QUERY_STR, &tree, code.as_bytes());
            helper.for_each_capture(|label, capture| {
                assert_eq!("function", label);
                assert_eq!(expected_name, super::function_definition_name(capture.node, code.as_bytes()));
            });
        }
    }
}
