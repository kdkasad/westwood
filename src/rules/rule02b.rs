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

use indoc::indoc;
use tree_sitter::QueryCapture;

use crate::{
    diagnostic::Diagnostic,
    helpers::{function_definition_name, QueryHelper},
    rules::api::Rule,
};

use crate::rules::api::SourceInfo;

use super::api::RuleDescription;

/// Number of lines per page
const PAGE_SIZE: usize = 61;
/// Maximum number of pages a function definition may span
const MAX_PAGES_PER_FUNCTION: usize = 2;

/// Tree-sitter query for Rule I:D.
const QUERY_STR: &str = indoc! {
    /* query */
    r"
    (function_definition) @function
    "
};

/// # Rule II:B.
///
/// See module-level documentation for details.
pub struct Rule02b {}

impl Rule for Rule02b {
    fn describe(&self) -> &'static RuleDescription {
        &RuleDescription {
            group_number: 2,
            letter: 'B',
            code: "II:B",
            name: "FunctionLength",
            description: "functions must be kept reasonably small",
        }
    }

    fn check<'a>(
        &self,
        SourceInfo {
            filename,
            tree,
            code,
            ..
        }: &'a SourceInfo,
    ) -> Vec<Diagnostic<'a>> {
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
                    let diagnostic = Diagnostic::new(self.describe(), message)
                        .with_violation_parts(
                            filename,
                            capture.node.into(),
                            format!(
                                "Function `{}()' is {} lines long",
                                function_definition_name(capture.node, code),
                                length
                            ),
                        );
                    diagnostics.push(diagnostic);
                }
            }
            _ => unreachable!(),
        });
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        diagnostic::Diagnostic,
        rules::api::{Rule, SourceInfo},
    };

    use pretty_assertions::assert_eq;

    use super::{Rule02b, MAX_PAGES_PER_FUNCTION, PAGE_SIZE};

    #[test]
    fn rule02b() {
        // Generate long function
        let mut code = String::new();
        code.push_str("int main() {\n");
        for _ in 0..(PAGE_SIZE * MAX_PAGES_PER_FUNCTION) {
            code.push_str("  (void) 0;\n");
        }
        code.push_str("}\n");

        // Test for diagnostic
        let rule02b = Rule02b {};
        let source = SourceInfo::new("", &code);
        assert_eq!(
            rule02b.check(&source),
            vec![Diagnostic::new(
                rule02b.describe(),
                format!(
                    "Functions must fit on {} pages, i.e. be no longer than {} lines",
                    MAX_PAGES_PER_FUNCTION,
                    PAGE_SIZE * MAX_PAGES_PER_FUNCTION
                )
            )
            .with_violation_parts(
                "",
                crate::diagnostic::SourceRange {
                    bytes: 0..(code.len() - 1),
                    start_pos: (0, 0),
                    end_pos: (code.lines().count(), 0)
                },
                format!(
                    "Function `main()' is {} lines long",
                    2 + MAX_PAGES_PER_FUNCTION * PAGE_SIZE
                )
            )]
        );
    }
}
