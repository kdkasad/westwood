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

//! # Rule II:A
//!
//! ```text
//!   A. Each line must be kept within 80 columns in order to make sure
//!      the entire line will fit on printouts.  If the line is too long,
//!      then it must be broken up into readable segments.
//!      The indentation of the code on the following lines needs to be
//!      at least 2 spaces.
//!
//!
//!      Example: room_temperature = list_head->left_node->
//!                                           left_node->
//!                                           left_node->
//!                                           left_node->
//!                                           left_node->
//!                                           temperature;
//!
//!      Example: fread(&value, sizeof(double),
//!                     1, special_fp);
//! ```
//!
//! # Implementation notes
//!
//! Currently, we only implement the 80-column limit and not the rule that wrapped statements must
//! be indented by 2 extra spaces.

use codespan_reporting::diagnostic::{Diagnostic, Label};
use tree_sitter::Tree;
use unicode_width::UnicodeWidthStr;

use crate::rules::api::Rule;

/// # Rule II:A.
///
/// See module-level documentation for details.
pub struct Rule2a {}

impl Rule for Rule2a {
    fn check(&self, _tree: &Tree, code_bytes: &[u8]) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();

        // Check for lines >80 columns long
        let code = std::str::from_utf8(code_bytes).expect("Code is not valid UTF-8");
        for (line, index) in LinesWithPosition::from(code) {
            let width = line_width(line);
            if width > 80 {
                let diagnostic = Diagnostic::warning()
                    .with_code("II:A")
                    .with_message("Line length exceeds 80 columns.")
                    .with_labels(vec![Label::primary((), (index + 80)..(index + line.len()))]);
                diagnostics.push(diagnostic);
            }
        }

        // TODO: Check indentation of wrapped lines

        diagnostics
    }
}

/// Iterator over the lines in a string while keeping track of the byte index within the source of
/// the start of each line.
struct LinesWithPosition<'a> {
    remaining_input: &'a str,
    index: usize,
}

impl<'a> From<&'a str> for LinesWithPosition<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            remaining_input: value,
            index: 0,
        }
    }
}

impl<'a> Iterator for LinesWithPosition<'a> {
    type Item = (&'a str, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_input.is_empty() {
            return None;
        }
        // TODO: Support \r\n line endings
        let start_index = self.index;
        let eol_index = self.remaining_input.find('\n').unwrap_or(self.remaining_input.len());
        let mut next_line_start = eol_index;
        if eol_index != self.remaining_input.len() {
            // Skip newline
            next_line_start += 1;
        }
        let line = &self.remaining_input[..eol_index];
        self.remaining_input = &self.remaining_input[next_line_start..];
        self.index += next_line_start;
        Some((line, start_index))
    }
}

/// Returns the width of a line in columns.
///
/// Returns the width according to the [unicode_width] module, but with tab characters (U+0009 or
/// `'\t'`) treated as 8 columns wide.
fn line_width(line: &str) -> usize {
    line.width() + line.chars().filter(|c| *c == '\t').count() * 7
}

#[cfg(test)]
mod tests {
    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    use pretty_assertions::assert_eq;

    #[test]
    fn line_width() {
        let tests = [
            ("", 0),
            ("\t", 8),
            ("\t\t", 16),
            ("\tint x;", 14),
            (
                "static void read_line(const char *restrict, char *restrict, size_t);",
                68,
            ),
            (
                "static void read_line(const char *restrict prompt, char *restrict buffer, size_t buffer_size);",
                94,
            ),
            ("int 😵 = 5;", 11),
        ];
        for (line, expected) in tests {
            assert_eq!(expected, super::line_width(line));
        }
    }
}
