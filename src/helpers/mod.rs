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

pub mod testing;

use tree_sitter::{
    Node, Query, QueryCapture, QueryCursor, QueryMatch, QueryPredicate, QueryPredicateArg, Range,
    StreamingIterator as _, Tree,
};
use unicode_width::UnicodeWidthChar;

/// Helper to handle creating and executing queries while handling custom predicates.
///
/// # Supported custom predicates
///
/// - `#has-ancestor?`: Takes a capture and a node kind (string) as operands and matches if the
///   captured node has an ancestor of the given kind.
/// - `#has-parent?`: Like `#has-ancestor?` but only checks the immediate parent of the captured
///   node, not all ancestors.
///
/// Each custom predicate also has a negated version prefixed with `not-`.
pub struct QueryHelper<'src> {
    query: Query,
    tree: &'src Tree,
    code: &'src [u8],
}

impl<'src> QueryHelper<'src> {
    /// Constructs a new [QueryHelper].
    /// This function does not execute the query.
    ///
    /// # Arguments
    ///
    /// - `query_src`: Tree-sitter query to execute.
    /// - `tree`: Tree to execute query on.
    /// - `code`: Source text/code that `tree` represents.
    #[must_use]
    pub fn new(query_src: &str, tree: &'src Tree, code: &'src [u8]) -> Self {
        let query =
            Query::new(&tree_sitter_c::LANGUAGE.into(), query_src).expect("Failed to parse query");
        Self { query, tree, code }
    }

    /// Returns a reference to this helper's query.
    pub fn query(&self) -> &Query {
        &self.query
    }

    /// Returns the index for the capture with the given name, or panics if there is no capture
    /// with such a name.
    pub fn expect_index_for_capture(&self, name: &str) -> u32 {
        self.query
            .capture_index_for_name(name)
            .unwrap_or_else(|| panic!("Query has no capture named `{}'", name))
    }

    /// Returns the node captured by the capture with the given index. To get an index from
    /// a capture name, use [`expect_index_for_capture()`][Self::expect_index_for_capture].
    ///
    /// Panics if the given capture does not have exactly one node.
    pub fn expect_node_for_capture_index(
        &self,
        qmatch: &QueryMatch<'_, 'src>,
        capture_index: u32,
    ) -> Node<'src> {
        let mut nodes = qmatch.nodes_for_capture_index(capture_index);
        let node = nodes.next().expect("Expected exactly one node for capture");
        assert!(nodes.next().is_none(), "Expected exactly one node for capture");
        node
    }

    /// Executes the query and calls a callback for each capture obtained by the query.
    ///
    /// # Arguments
    ///
    /// - `handler`: Callback to execute for each capture.
    ///   The arguments to the callback are the name of the capture and the [QueryCapture].
    pub fn for_each_capture<'a, F>(&'a self, mut handler: F)
    where
        F: FnMut(&'a str, QueryCapture<'a>),
    {
        let mut cursor = QueryCursor::new();
        let capture_names = self.query.capture_names();
        let mut captures = cursor.captures(&self.query, self.tree.root_node(), self.code);
        while let Some((qmatch, capture_index_within_match)) = captures.next() {
            let custom_predicates = self.query.general_predicates(qmatch.pattern_index);
            if !custom_predicates.iter().all(|pred| self.predicate_matches(pred, qmatch)) {
                continue;
            }
            let capture = qmatch.captures[*capture_index_within_match];
            let name = capture_names[capture.index as usize];
            handler(name, capture);
        }
    }

    /// Executes the query and calls a callback for each match obtained by the query.
    ///
    /// # Arguments
    ///
    /// - `handler`: Callback to execute for each match.
    ///   The argument to the callback is the [QueryMatch].
    pub fn for_each_match<F>(&self, mut handler: F)
    where
        F: FnMut(&QueryMatch),
    {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, self.tree.root_node(), self.code);
        while let Some(qmatch) = matches.next() {
            let custom_predicates = self.query.general_predicates(qmatch.pattern_index);
            if !custom_predicates.iter().all(|pred| self.predicate_matches(pred, qmatch)) {
                continue;
            }
            handler(qmatch);
        }
    }

    /// Checks if a custom predicate matches.
    ///
    /// # Arguments
    ///
    /// - `predicate`: [`QueryPredicate`] to match.
    /// - `qmatch`: [`QueryMatch`] containing predicate.
    fn predicate_matches(&self, predicate: &QueryPredicate, qmatch: &QueryMatch) -> bool {
        let orig_op: &str = predicate.operator.as_ref();

        // If the operator starts with #not- then we simply perform the regular operator and negate
        // it when returning.
        let (op, negate) =
            orig_op.strip_prefix("not-").map_or((orig_op, false), |rest| (rest, true));

        let result = match op {
            // Matches if any ancestor of the captured node is of the given kind
            "has-ancestor?" => {
                if let [QueryPredicateArg::Capture(capture_index), QueryPredicateArg::String(kind)] =
                    predicate.args.as_ref()
                {
                    // Starting at the root, descend until we reach the captured node (target) and
                    // check if any of the nodes along the way are of the given kind.
                    let mut captured_nodes = qmatch.nodes_for_capture_index(*capture_index);
                    let target = captured_nodes.next().expect("Expected one captured node");
                    assert!(
                        captured_nodes.next().is_none(),
                        "Expected no more than one captured node"
                    );
                    let mut maybe_node = Some(self.tree.root_node());
                    let mut found = false;
                    while let Some(node) = maybe_node {
                        if node.id() == target.id() {
                            break;
                        }
                        if node.kind() == kind.as_ref() {
                            found = true;
                            break;
                        }
                        maybe_node = node.child_with_descendant(target);
                    }
                    found
                } else {
                    panic!("Invalid arguments to #{}. Expected a capture and a string.", orig_op);
                }
            }

            // Matches if the node's immediate parent is of the given kind
            "has-parent?" => {
                if let [QueryPredicateArg::Capture(capture_index), QueryPredicateArg::String(kind)] =
                    predicate.args.as_ref()
                {
                    // Starting at the root, descend until we reach the captured node (target) and
                    // check if any of the nodes along the way are of the given kind.
                    let mut captured_nodes = qmatch.nodes_for_capture_index(*capture_index);
                    let target = captured_nodes.next().expect("Expected one captured node");
                    assert!(
                        captured_nodes.next().is_none(),
                        "Expected no more than one captured node"
                    );
                    target.parent().is_some_and(|parent| parent.kind() == kind.as_ref())
                } else {
                    panic!("Invalid arguments to #{}. Expected a capture and a string.", orig_op);
                }
            }

            _ => {
                eprintln!("WARNING: Ignoring unknown predicate `{}'", orig_op);
                false
            }
        };
        result ^ negate
    }
}

/// Returns the name of a function defined by a `function_definition` node.
///
/// # Panics
///
/// This function panics if:
/// - the given `node`'s [kind][Node::kind()] is not `function_definition`;
/// - the given `node` does not have an `identifier` child reachable by repeatedly traversing to
///   the node named by the `declarator` field;
/// - the node's text is not valid UTF-8
///
pub fn function_definition_name<'code>(node: Node, code: &'code [u8]) -> &'code str {
    assert_eq!(
        "function_definition",
        node.kind(),
        "Expected node to have kind `function_definition'"
    );

    let mut node = node;
    while node.kind() != "identifier" {
        node = node
            .child_by_field_name("declarator")
            .expect("Expected node to have a `declarator' field");
    }
    node.utf8_text(code).expect("Code is not valid UTF-8")
}

/// Gets the number of columns by which this line is indented. Tab characters (U+0009 or `'\t'`)
/// are counted as 8 columns. All other whitespace is sized using [unicode_width].
pub fn indent_width(line: &str) -> usize {
    line.chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| match c {
            '\t' => 8,
            // SAFETY: We're filtering for only whitespace, so we won't get a control character, which
            // is when .width() returns None.
            other => other.width().unwrap(),
        })
        .sum()
}

/// Iterator over the lines in a string while keeping track of the byte index within the source of
/// the start of each line.
pub struct LinesWithPosition<'a> {
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

/// Transforms an iterator over [ranges][Range] by collapsing adjacent ranges.
///
/// For example (using simple numeric ranges), the input:
/// ```text
/// 1..2, 2..3, 3..5, 6..7, 8..9, 9..10
/// ```
/// would be transformed by the [RangeCollapser] into:
/// ```text
/// 1..5, 6..7, 8..10
/// ```
#[derive(Clone, Debug)]
pub struct RangeCollapser<I: Iterator<Item = Range>> {
    src: I,
    current_range: Option<Range>,
}

impl<I, J> From<J> for RangeCollapser<I>
where
    I: Iterator<Item = Range>,
    J: IntoIterator<IntoIter = I>,
{
    fn from(value: J) -> Self {
        Self {
            src: value.into_iter(),
            current_range: None,
        }
    }
}

impl<I: Iterator<Item = Range>> Iterator for RangeCollapser<I> {
    type Item = Range;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.src.next() {
                // If there is no next range, return the current range.
                None => return self.current_range.take(),
                Some(next) => match self.current_range.as_mut() {
                    Some(current) => {
                        if current.end_point == next.start_point {
                            // If adjacent, update the current range's end point.
                            current.end_point = next.end_point;
                            current.end_byte = next.end_byte;
                        } else {
                            // If not adjacent, return the old range and start tracking from the
                            // next range.
                            return self.current_range.replace(next);
                        }
                    }
                    None => self.current_range = Some(next),
                },
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::process::ExitCode;

    use super::{testing::test_captures, QueryHelper, RangeCollapser};

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use tree_sitter::{Parser, Range};

    #[test]
    /// Test the `#has-ancestor?` custom predicate.
    fn test_has_ancestor() -> ExitCode {
        let input = indoc! { /* c */ r#"
            int a;
                //!? outfunc
            int b = 0;
                //!? outfunc
            int func() {
                //!? infunc
                int c;
                    //!? infunc
                if (a == b) {
                    //!? infunc inif
                         //!? infunc inif
                    int d;
                        //!? infunc inif
                    return d;
                           //!? infunc inif
                }
            }
        "# };
        let query = indoc! { /* query */ r#"
            ((identifier) @infunc
                (#has-ancestor? @infunc function_definition))
            ((identifier) @outfunc
                (#not-has-ancestor? @outfunc function_definition))
            ((identifier) @inif
                (#has-ancestor? @inif if_statement))
        "# };
        test_captures(query, input)
    }

    #[test]
    /// Test the `#has-parent?` custom predicate.
    fn test_has_parent() -> ExitCode {
        let input = indoc! { /* c */ r#"
            int a = 0;
            //!? toplevel
                    //!? number

            int main() {
            //!? toplevel
                //!? funcdeclname
                return 0;
            }
        "# };

        let query = indoc! { /* query */ r#"
            ((_) @toplevel
                (#has-parent? @toplevel translation_unit))
            ((_ declarator: (identifier) @funcdeclname)
                (#has-parent? @funcdeclname function_declarator))
            ((number_literal) @number
                (#not-has-parent? @number return_statement))
        "# };
        test_captures(query, input)
    }

    #[test]
    /// Test [function_definition_name()][super::function_definition_name()].
    fn function_definition_name() {
        // List of tuples of the form (code, function name)
        let tests = [
            ("int main() {}", "main"),
            ("void **(*ptrptrptr)(char a[])", "ptrptrptr"),
            ("char *strcpy(char *dst, const char *src)", "strcpy"),
            ("char *strdup(const char *src)", "strdup"),
            ("void free(void *ptr)", "free"),
        ];
        for (code, expected_name) in tests {
            let mut parser = Parser::new();
            parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
            let tree = parser.parse(code.as_bytes(), None).unwrap();
            let helper =
                QueryHelper::new("(function_definition) @function", &tree, code.as_bytes());
            helper.for_each_capture(|label, capture| {
                assert_eq!("function", label);
                assert_eq!(
                    expected_name,
                    super::function_definition_name(capture.node, code.as_bytes())
                );
            });
        }
    }

    #[test]
    /// Test [indent_width()][super::indent_width()].
    fn indent_width() {
        let tests = [
            ("a", 0),
            (" a", 1),
            ("  a", 2),
            ("\ta", 8),
            (" \t a", 10),
            (" ", 1),
            ("\t", 8),
        ];
        for (line, expected_indent) in tests {
            assert_eq!(expected_indent, super::indent_width(line));
        }
    }

    #[test]
    fn range_collapser() {
        let code = indoc! {
            /* c */ r#"
            #define A 1
            #define B 2

            #define C 1
            #define D 2
            "#
        };
        let mut ranges: Vec<Range> = Vec::with_capacity(4);
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();
        let tree = parser.parse(code.as_bytes(), None).unwrap();
        let helper = QueryHelper::new("(preproc_def) @define", &tree, code.as_bytes());
        helper.for_each_capture(|_label, capture| ranges.push(capture.node.range()));
        let mut collapser = RangeCollapser::from(ranges.clone());
        let group1 = collapser.next().unwrap();
        let group2 = collapser.next().unwrap();
        assert!(collapser.next().is_none());
        assert_eq!(ranges[0].start_byte, group1.start_byte);
        assert_eq!(ranges[0].start_point, group1.start_point);
        assert_eq!(ranges[1].end_byte, group1.end_byte);
        assert_eq!(ranges[1].end_point, group1.end_point);
        assert_eq!(ranges[2].start_byte, group2.start_byte);
        assert_eq!(ranges[2].start_point, group2.start_point);
        assert_eq!(ranges[3].end_byte, group2.end_byte);
        assert_eq!(ranges[3].end_point, group2.end_point);
    }
}
