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
//! This is almost impossible to check programmatically, so [`Rule1b`] does nothing. It (and this
//! module) are included here for the sake of completeness.
//!
//! # To do
//!
//! - Make this rule produce a table of all declared identifiers at the end of parsing.

use tree_sitter::Tree;

use crate::rules::api::Rule;

/// # Rule I:B.
///
/// See module-level documentation for details.
pub struct Rule1b {}

impl Rule for Rule1b {
    fn check(&self, _filename: &str, _tree: &Tree, _code: &[u8]) {
        return;
    }
}
