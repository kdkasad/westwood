//! API for [rules][Rule].

use tree_sitter::Tree;

/// Represents a linter rule.
pub trait Rule {
    /// Checks a source file for compliance with this rule.
    ///
    /// # Arguments
    ///
    /// - `filename`: Name of the file being checked.
    /// - `tree`: [`Tree`] representing the file.
    /// - `code`: Text/code of the given file.
    fn check(&self, filename: &str, tree: &Tree, code: &[u8]);
}
