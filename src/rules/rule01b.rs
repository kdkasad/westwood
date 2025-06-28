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

//! # Rule I:B
//!
//! ```text
//!   B. Use descriptive and meaningful names.
//!
//!      Example: Variable such as "room_temperature" is
//!      descriptive and meaningful, but "i" is not.  An exception can
//!      be made if "i" is used for loop counting, array indexing, etc.
//!
//!      An exception can also be made if the variable name is something
//!      commonly used in a mathematical equation, and the code is
//!      implementing that equation.
//! ```
//!
//! # Implementation notes
//!
//! This is almost impossible to check programmatically, so [`Rule01b`] does nothing. It (and this
//! module) are included here for the sake of completeness.
//!
//! # To do
//!
//! - Make this rule produce a table of all declared identifiers at the end of parsing.

use crate::{
    diagnostic::Diagnostic,
    rules::api::{Rule, RuleDescription, SourceInfo},
};

/// # Rule I:B.
///
/// See module-level documentation for details.
pub struct Rule01b {}

impl Rule for Rule01b {
    fn describe(&self) -> &'static RuleDescription {
        &RuleDescription {
            group_number: 1,
            letter: 'B',
            code: "I:B",
            name: "MeaningfulNames",
            description: "variable names must be descriptive and meaningful",
        }
    }

    fn check<'a>(&self, _: &'a SourceInfo) -> Vec<Diagnostic<'a>> {
        Vec::new()
    }
}
