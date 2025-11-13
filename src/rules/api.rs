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

//! API for [rules][Rule].

use tree_sitter::Tree;

use crate::{diagnostic::Diagnostic, helpers::LinesWithPosition};

#[derive(Debug, Clone)]
pub struct SourceInfo<'src> {
    pub filename: &'src str,
    pub tree: Tree,
    pub code: &'src str,
    pub lines: Box<[(&'src str, usize)]>,
}

impl<'src> SourceInfo<'src> {
    pub fn new(filename: &'src str, code: &'src str) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .expect("Failed to set language");
        let tree = parser.parse(code, None).expect("Failed to parse code");
        let lines = LinesWithPosition::from(code).collect();
        Self {
            filename,
            tree,
            code,
            lines,
        }
    }
}

/// Describes a linter rule. Used for adding metadata about the rule to a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleDescription {
    /// Number of the group this rule belongs to.
    pub group_number: u8,
    /// Letter of the rule within its group.
    pub letter: char,
    /// Rule code (e.g. `"III:F"`).
    pub code: &'static str,
    /// Name of the rule in Pascal case (e.g. `"MeaningfulNames"`).
    pub name: &'static str,
    /// Description of the rule, ideally in the form "X must (be) Y".
    pub description: &'static str,
}

impl Ord for RuleDescription {
    /// Compares two rule descriptions based on their group number and letter.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_index = (self.group_number, self.letter);
        let other_index = (other.group_number, other.letter);
        self_index.cmp(&other_index)
    }
}

impl PartialOrd for RuleDescription {
    /// Defers to [`RuleDescription::cmp()`].
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<R: Rule> From<&R> for &'static RuleDescription {
    /// Converts a [rule][Rule] into its [description][RuleDescription].
    fn from(rule: &R) -> &'static RuleDescription {
        rule.describe()
    }
}

/// Represents a linter rule.
pub trait Rule {
    /// Checks a source file for compliance with this rule.
    ///
    /// # Arguments
    ///
    /// - `filename`: Name of the file being checked.
    /// - `tree`: [`Tree`] representing the file.
    /// - `code`: Text/code of the given file.
    #[must_use]
    fn check<'a>(&self, source: &'a SourceInfo) -> Vec<Diagnostic<'a>>;

    /// Returns a static description of the rule.
    #[must_use]
    fn describe(&self) -> &'static RuleDescription;

    /// Creates a new diagnostic with the rule's description and a message.
    #[must_use]
    fn report<'a>(&self, message: &'a str) -> Diagnostic<'a> {
        Diagnostic::new(self.describe(), message)
    }
}
