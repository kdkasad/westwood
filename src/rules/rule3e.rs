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

//! # Rule III:E
//!
//! ```text
//!    C. Never put trailing whitespace at the end of a line.
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use tree_sitter::Tree;

use crate::{helpers::LinesWithPosition, rules::api::Rule};

/// # Rule III:E.
///
/// See module-level documentation for details.
pub struct Rule3e {}

impl Rule for Rule3e {
    fn check(&self, _tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();
        let code_str = std::str::from_utf8(code).expect("Code is not valid UTF-8");
        for (line, index) in LinesWithPosition::from(code_str) {
            let trimmed_line = line.trim_end();
            if trimmed_line.len() != line.len() {
                // Start/end of trailing whitespace
                let start = index + trimmed_line.len();
                let end = index + line.len();
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("III:E")
                        .with_message("Line contains trailing whitespace")
                        .with_label(Label::primary((), start..end)),
                );
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use tree_sitter::Parser;

    use crate::rules::api::Rule;

    use super::Rule3e;

    /// This test is very basic and just checks the number of diagnostics produced. It doesn't
    /// check the ranges labeled or the message(s).
    #[test]
    fn rule3e() {
        let code = "int main() { \n  return 0;\t\n}\n";
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        let rule = Rule3e {};
        let diagnostics = rule.check(&tree, code.as_bytes());
        assert_eq!(2, diagnostics.len());
    }
}
