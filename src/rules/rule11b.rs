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

//! # Rule XI:B
//!
//! ```text
//!    B. Use only UNIX newline encoding (\n). DOS newlines (\r\n) are prohibited.
//! ```

use std::num::NonZeroUsize;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use tree_sitter::Tree;

use crate::rules::api::Rule;

/// # Rule XI:B.
///
/// See module-level documentation for details.
pub struct Rule11b {
    max_diagnostics: Option<NonZeroUsize>,
}

impl Rule11b {
    /// Constructs a new instance of this rule.
    ///
    /// `max_diagnostics` specifies the maximum number of diagnostics to output. If more than this
    /// are produced, a note is displayed on the last one and the rest are hidden.
    pub fn new(max_diagnostics: Option<NonZeroUsize>) -> Self {
        Self { max_diagnostics }
    }
}

impl Rule for Rule11b {
    fn check(&self, _tree: &Tree, code: &[u8]) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();

        // Search for DOS-style newlines
        let code_str = std::str::from_utf8(code).expect("Code is not valid UTF-8");
        // Split on newlines, keeping track of the position within the source
        let mut next_line_start_pos = 0;
        let mut dos_lines = code_str
            .split('\n')
            .map(|line| {
                let cur_line_start_pos = next_line_start_pos;
                next_line_start_pos += line.len() + 1;
                (line, cur_line_start_pos)
            })
            .filter(|(line, _pos)| line.ends_with('\r'));

        // Produce diagnostics
        for (line, start_pos) in dos_lines.by_ref() {
            // Position of '\r' in line
            let cr_pos = start_pos + line.len() - 1;
            diagnostics.push(
                Diagnostic::warning()
                    .with_code("XI:B")
                    .with_message("Line contains DOS-style ending")
                    .with_label(Label::primary((), cr_pos..(cr_pos + 1)))
                    // TODO: Don't hard-code escape sequences
                    .with_note(
                        "See \x1b[4m:h 'fileformat'\x1b[m in Vim for info on how to fix this",
                    ),
            );

            // Apply the limit on the number of diagnostics produced
            if self.max_diagnostics.is_some_and(|max| diagnostics.len() == max.get()) {
                // SAFETY: We know diagnostics will have a last element because if
                // self.max_diagnostics is some, its value cannot be zero.
                diagnostics.last_mut().unwrap().notes.push(format!(
                    "{} more lines contain DOS endings, but those warnings are suppressed to avoid noise.",
                    dos_lines.count()
                ));
                break;
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.
}
