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

use crate::rules::api::Rule;

use crate::rules::api::SourceInfo;

use super::api::RuleDescription;

/// # Rule III:E.
///
/// See module-level documentation for details.
pub struct Rule03e {}

impl Rule for Rule03e {
    fn describe(&self) -> &'static RuleDescription {
        &RuleDescription {
            group_number: 3,
            letter: 'E',
            code: "III:E",
            name: "TrailingWhitespace",
            description: "lines must not have trailing whitespace",
        }
    }

    fn check(&self, SourceInfo { lines, .. }: &SourceInfo) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();
        for (line, index) in lines {
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
    use crate::rules::api::{Rule, SourceInfo};

    use super::Rule03e;

    /// This test is very basic and just checks the number of diagnostics produced. It doesn't
    /// check the ranges labeled or the message(s).
    #[test]
    fn rule03e() {
        let code = "int main() { \n  return 0;\t\n}\n";
        let source = SourceInfo::new(code);
        let rule = Rule03e {};
        let diagnostics = rule.check(&source);
        assert_eq!(2, diagnostics.len());
    }
}
