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


//! # Rule I:A
//!
//! ```text
//!    A. Variable names should be in all lowercase.
//!       If the name is composed of more than one word, then underscores
//!       must be used to separate them.
//!
//!       Example: int temperature = 0;
//!
//!       Example: int room_temperature = 0;
//! ```
//!
//! # Implementation notes
//!
//! This rule checks that all declared identifiers are in lowercase. It cannot check whether
//! underscores are used to separate words because splitting an identifier into words is
//! subjective.

use indoc::indoc;
use tree_sitter::Tree;

use crate::{helpers::QueryHelper, rules::api::Rule};

const QUERY_STR: &'static str = indoc! { /* query */ r#"
    (
        (_ declarator: (identifier) @name)
        (#match? @name "[A-Z]")
    )
"# };

/// # Rule I:A.
///
/// See module-level documentation for details.
pub struct Rule1a {}

impl Rule for Rule1a {
    fn check(&self, filename: &str, tree: &Tree, code: &[u8]) {
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        helper.for_each_capture(|_label, capture| {
            let start = capture.node.start_position();
            let variable_name = capture
                .node
                .utf8_text(code)
                .expect("Code is not valid UTF-8");
            println!(
                "{}:{}:{}: Name \"{}\" must be in lower snake case.",
                filename, start.row, start.column, variable_name
            );
        });
    }
}
