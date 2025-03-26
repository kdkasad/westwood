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

//! # Rule XI:A
//!
//! ```text
//!    A. Do not use tabs for indentation.
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use tree_sitter::Tree;

use crate::{helpers::LinesWithPosition, rules::api::Rule};

/// # Rule XI:A.
///
/// See module-level documentation for details.
pub struct Rule11a {
    max_diagnostics: Option<usize>,
}

impl Rule11a {
    /// Constructs a new instance of this rule.
    ///
    /// `max_diagnostics` specifies the maximum number of diagnostics to output. If more than this
    /// are produced, a note is displayed on the last one and the rest are hidden.
    #[must_use]
    pub fn new(max_diagnostics: Option<usize>) -> Self {
        Self { max_diagnostics }
    }
}

impl Rule for Rule11a {
    fn check(&self, _tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();

        let lines =
            LinesWithPosition::from(std::str::from_utf8(code).expect("Code is not valid UTF-8"));
        for (line, start_pos) in lines {
            // Get just the part of the line which consists of indentation
            let indentation = &line[..(line.len() - line.trim_start().len())];
            if indentation.is_empty() {
                continue;
            }

            if indentation.as_bytes().iter().all(|c| *c == b'\t') {
                // If the whole indentation string consists of tabs, then just label the whole
                // thing.
                diagnostics.push(
                    Diagnostic::warning()
                        .with_code("XI:A")
                        .with_message("Use spaces instead of tabs for indentation")
                        .with_label(
                            Label::primary((), start_pos..(start_pos + indentation.len()))
                                .with_message("Indentation uses tabs"),
                        ),
                );
            } else {
                // If there is a mix of tabs and non-tabs, label each tab separately
                let mut labels = line
                    .char_indices()
                    .take_while(|(_pos, c)| c.is_whitespace())
                    .filter(|(_pos, c)| *c == '\t')
                    .map(|(pos, _c)| pos)
                    .map(|pos| {
                        #[allow(clippy::range_plus_one)]
                        Label::primary((), (start_pos + pos)..(start_pos + pos + 1))
                            .with_message("Tab character found here")
                    })
                    .peekable();
                if labels.peek().is_some() {
                    diagnostics.push(
                        Diagnostic::warning()
                            .with_code("XI:A")
                            .with_message("Use spaces instead of tabs for indentation")
                            .with_notes(vec!["Line mixes spaces and tabs".to_string()])
                            .with_labels_iter(labels),
                    );
                }
            }
        }

        // Apply the limit on the number of diagnostics produced
        if let Some(max) = self.max_diagnostics {
            if diagnostics.len() >= max {
                let remaining = diagnostics.len() - max;
                diagnostics.truncate(max);
                diagnostics.last_mut().unwrap().notes.push(format!(
                    "{remaining} more lines contain tabs, but those warnings are suppressed to avoid noise."
                ));
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    use pretty_assertions::assert_eq;
    use tree_sitter::{Parser, Tree};

    use crate::rules::api::Rule;

    /// Returns a [Tree] for the given C code.
    fn parse(code: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
        parser.parse(code.as_bytes(), None).unwrap()
    }

    /// Tests when lines contain only tabs for indentation.
    #[test]
    fn all_tabs() {
        let code = "#include <stdio.h>\nint main() {\n\t\tprintf(\"Hello, world!\\n\");\n\t\treturn 0;\n}\n";
        let rule = super::Rule11a::new(None);
        let diagnostics = rule.check(&parse(code), code.as_bytes());
        assert_eq!(2, diagnostics.len());
        assert!(diagnostics.iter().all(|diag| diag.labels.len() == 1));
    }

    /// Tests when lines contain a mix of tabs and spaces.
    #[test]
    fn mix_tabs_spaces() {
        let code = "#include <stdio.h>\nint main() {\n  \tprintf(\"Hello, world!\\n\");\n  \treturn 0;\n}\n";
        let rule = super::Rule11a::new(None);
        let diagnostics = rule.check(&parse(code), code.as_bytes());
        assert_eq!(2, diagnostics.len());
        assert!(diagnostics.iter().all(|diag| diag.labels.len() == 1));
        assert!(diagnostics.iter().all(|diag| diag.notes.len() == 1));
    }

    /// Tests when lines don't use tabs (checks for false positives).
    #[test]
    fn no_tabs() {
        let code =
            "#include <stdio.h>\nint main() {\n  printf(\"Hello, world!\\n\");\n  return 0;\n}\n";
        let rule = super::Rule11a::new(None);
        let diagnostics = rule.check(&parse(code), code.as_bytes());
        assert!(diagnostics.is_empty());
    }
}
