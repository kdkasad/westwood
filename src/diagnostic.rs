//! Diagnostic types

use std::{borrow::Cow, ops::Range};

use tree_sitter::Node;

use crate::{
    helpers::line_width,
    rules::api::{RuleDescription, SourceInfo},
};

/// Represents a diagnostic message for a code standard violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic<'a> {
    /// Description of the code standard rule that was violated.
    pub rule: &'static RuleDescription,

    /// Message describing the violation.
    pub message: Cow<'a, str>,

    /// Locations of code that violated the rule.
    pub violations: Vec<Span<'a>>,

    /// Locations of code that are relevant to the violation but are not violations themselves.
    pub references: Vec<Span<'a>>,

    /// Optional suggestion for fixing the violation.
    pub suggestion: Option<String>,
}

impl<'a> Diagnostic<'a> {
    /// Creates a new `Diagnostic` with the given rule, message, and spans.
    pub fn new<M>(rule: &'static RuleDescription, message: M) -> Self
    where
        M: Into<Cow<'a, str>>,
    {
        Self {
            rule,
            message: message.into(),
            violations: Vec::new(),
            references: Vec::new(),
            suggestion: None,
        }
    }

    /// Adds a violation span and returns the modified diagnostic.
    pub fn with_violation(mut self, span: Span<'a>) -> Self {
        self.violations.push(span);
        self
    }

    /// Adds a violation span constructed by calling [`Span::new()`] with the given arguments.
    /// Returns the modified diagnostic.
    pub fn with_violation_parts<L>(
        mut self,
        filename: &'a str,
        range: SourceRange,
        label: L,
    ) -> Self
    where
        L: Into<Cow<'a, str>>,
    {
        self.violations.push(Span::new(filename, range, label));
        self
    }

    /// Replaces the existing violations with the provided list and returns the modified
    /// diagnostic.
    pub fn with_violations(mut self, violations: Vec<Span<'a>>) -> Self {
        self.violations = violations;
        self
    }

    /// Adds a reference span and returns the modified diagnostic.
    pub fn with_reference(mut self, span: Span<'a>) -> Self {
        self.references.push(span);
        self
    }

    /// Adds a reference span constructed by calling [`Span::new()`] with the given arguments.
    /// Returns the modified diagnostic.
    pub fn with_reference_parts<L>(
        mut self,
        filename: &'a str,
        range: SourceRange,
        label: L,
    ) -> Self
    where
        L: Into<Cow<'a, str>>,
    {
        self.references.push(Span::new(filename, range, label));
        self
    }

    /// Replaces the existing references with the provided list and returns the modified
    /// diagnostic.
    pub fn with_references(mut self, references: Vec<Span<'a>>) -> Self {
        self.references = references;
        self
    }

    /// Adds a suggestion and returns the modified diagnostic.
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span<'a> {
    /// Name of the source file.
    pub filename: &'a str,

    /// Range of the source code.
    pub range: SourceRange,

    /// Label for the span.
    pub label: Cow<'a, str>,
}

impl<'a> Span<'a> {
    /// Creates a new `Span` with the given filename, range, and label.
    pub fn new<L>(filename: &'a str, range: SourceRange, label: L) -> Self
    where
        L: Into<Cow<'a, str>>,
    {
        Self {
            filename,
            range,
            label: label.into(),
        }
    }
}

/// Represents a range of source code, as both a byte range and line/column positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRange {
    /// Range of bytes in the source code.
    pub bytes: Range<usize>,
    /// Start position (inclusive) in the source code as 0-indexed (row, column).
    pub start_pos: (usize, usize),
    /// End position (exclusive) in the source code as 0-indexed (row, column).
    pub end_pos: (usize, usize),
}

impl From<Node<'_>> for SourceRange {
    /// Creates a [`SourceRange`] from a [`Node`]'s range.
    fn from(node: Node<'_>) -> Self {
        Self {
            bytes: node.start_byte()..node.end_byte(),
            start_pos: (node.start_position().row, node.start_position().column),
            end_pos: (node.end_position().row, node.end_position().column),
        }
    }
}

impl SourceRange {
    /// Creates a new [`SourceRange`] that starts at the start of the first node and ends at the
    /// start of the second node.
    pub fn start_to_start(left: Node<'_>, right: Node<'_>) -> Self {
        Self {
            bytes: left.start_byte()..right.start_byte(),
            start_pos: (left.start_position().row, left.start_position().column),
            end_pos: (right.start_position().row, right.start_position().column),
        }
    }

    // Generate the function above but for end_to_end, start_to_end, and end_to_start
    /// Creates a new [`SourceRange`] that starts at the end of the first node and ends at the
    /// end of the second node.
    pub fn end_to_end(left: Node<'_>, right: Node<'_>) -> Self {
        Self {
            bytes: left.end_byte()..right.end_byte(),
            start_pos: (left.end_position().row, left.end_position().column),
            end_pos: (right.end_position().row, right.end_position().column),
        }
    }

    /// Creates a new [`SourceRange`] that starts at the start of the first node and ends at the
    /// end of the second node.
    pub fn start_to_end(left: Node<'_>, right: Node<'_>) -> Self {
        Self {
            bytes: left.start_byte()..right.end_byte(),
            start_pos: (left.start_position().row, left.start_position().column),
            end_pos: (right.end_position().row, right.end_position().column),
        }
    }

    /// Creates a new [`SourceRange`] that starts at the end of the first node and ends at the
    /// start of the second node.
    pub fn end_to_start(left: Node<'_>, right: Node<'_>) -> Self {
        Self {
            bytes: left.end_byte()..right.start_byte(),
            start_pos: (left.end_position().row, left.end_position().column),
            end_pos: (right.start_position().row, right.start_position().column),
        }
    }

    /// Creates a [`SourceRange`] from just a byte range, using the provided `SourceInfo` to determine
    /// the start and end positions.
    pub fn from_byte_range(bytes: Range<usize>, source: &SourceInfo) -> Self {
        // Find start line
        let start_line_i = source
            .lines
            .partition_point(|&(_, pos)| pos <= bytes.start)
            .checked_sub(1)
            .unwrap();
        let start_line_pos = source.lines[start_line_i].1;
        // Select the part of the line that is within the range to calculate the width
        let start_line_selected_part =
            &source.lines[start_line_i].0[..(bytes.start - start_line_pos)];

        // Find end line
        let end_line_i = source
            .lines
            .partition_point(|&(_, pos)| pos <= bytes.end)
            .checked_sub(1)
            .unwrap();
        let end_line_pos = source.lines[end_line_i].1;
        // Select the part of the line that is within the range to calculate the width
        let end_line_selected_part = &source.lines[end_line_i].0[..(bytes.end - end_line_pos)];

        Self {
            bytes,
            start_pos: (start_line_i, line_width(start_line_selected_part)),
            end_pos: (end_line_i, line_width(end_line_selected_part)),
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::{SourceInfo, SourceRange};

    use pretty_assertions::assert_eq;

    #[test]
    fn source_range_with_pos_from() {
        let code = indoc! { /* c */ r#"
            int main() {
                return 0;
            } /* main() */
        "# };
        let source = SourceInfo::new("test.c", code);
        let tests = vec![
            (0..3, (0, 0), (0, 3)),   // "int"
            (0..12, (0, 0), (0, 12)), // first line without newline
            (0..13, (0, 0), (1, 0)),  // first line with newline
        ];
        for (byte_range, expected_start, expected_end) in tests.into_iter() {
            let source_range = SourceRange::from_byte_range(byte_range, &source);
            assert_eq!(source_range.start_pos, expected_start);
            assert_eq!(source_range.end_pos, expected_end);
        }
    }
}
