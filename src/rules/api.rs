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

use codespan_reporting::diagnostic::Diagnostic;
use tree_sitter::Tree;

use crate::helpers::LinesWithPosition;

pub struct SourceInfo<'src> {
    pub tree: Tree,
    pub code: &'src str,
    pub lines: Box<[(&'src str, usize)]>,
}

impl<'src> SourceInfo<'src> {
    pub fn new(code: &'src str) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .expect("Failed to set language");
        let tree = parser.parse(code, None).expect("Failed to parse code");
        let lines = LinesWithPosition::from(code).collect();
        Self { tree, code, lines }
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
    fn check(&self, source: &SourceInfo) -> Vec<Diagnostic<()>>;
}
