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

//! # Rule XI:E
//!
//! ```text
//!    E. The use of goto is forbidden in this course.
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;

use crate::{helpers::QueryHelper, rules::api::Rule};

use crate::rules::api::SourceInfo;

/// Tree-sitter query for Rule XI:E.
const QUERY_STR: &str = indoc! {
    /* query */
    r"
    (goto_statement) @goto
    "
};

/// # Rule XI:E.
///
/// See module-level documentation for details.
pub struct Rule11e {}

impl Rule for Rule11e {
    fn check(&self, SourceInfo { tree, code, .. }: &SourceInfo) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        helper.for_each_capture(|label, capture| {
            assert_eq!("goto", label);
            diagnostics.push(
                Diagnostic::warning()
                    .with_code("XI:E")
                    .with_message("Do not use `goto'")
                    .with_label(Label::primary((), capture.node.byte_range())),
            );
        });
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use std::process::ExitCode;

    use indoc::indoc;

    use crate::helpers::testing::test_captures;

    use super::QUERY_STR;

    #[test]
    fn rule11e_captures() -> ExitCode {
        let input = indoc! {
            /* c */ r"
            int main() {
                goto label;
                //!? goto
                label:
                return 0;
            }
            "
        };
        test_captures(QUERY_STR, input)
    }
}
