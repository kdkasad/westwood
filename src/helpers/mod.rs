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
    Query, QueryCapture, QueryCursor, QueryMatch, QueryPredicate, QueryPredicateArg,
    StreamingIterator as _, Tree,
};

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

    /// Executes the query and calls a callback for each capture obtained by the query.
    ///
    /// # Arguments
    ///
    /// - `handler`: Callback to execute for each capture.
    ///   The arguments to the callback are the name of the capture and the [QueryCapture].
    pub fn for_each_capture<F>(&self, mut handler: F)
    where
        F: FnMut(&str, QueryCapture),
    {
        let mut cursor = QueryCursor::new();
        let capture_names = self.query.capture_names();
        let mut captures = cursor.captures(&self.query, self.tree.root_node(), self.code);
        while let Some((qmatch, capture_index_within_match)) = captures.next() {
            let custom_predicates = self.query.general_predicates(qmatch.pattern_index);
            if !custom_predicates
                .iter()
                .all(|pred| self.predicate_matches(pred, qmatch))
            {
                continue;
            }
            let capture = qmatch.captures[*capture_index_within_match];
            let name = capture_names[capture.index as usize];
            handler(name, capture);
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
        let (op, negate) = orig_op
            .strip_prefix("not-")
            .map_or((orig_op, false), |rest| (rest, true));

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
                    panic!(
                        "Invalid arguments to #{}. Expected a capture and a string.",
                        orig_op
                    );
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
                    target
                        .parent()
                        .is_some_and(|parent| parent.kind() == kind.as_ref())
                } else {
                    panic!(
                        "Invalid arguments to #{}. Expected a capture and a string.",
                        orig_op
                    );
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

#[cfg(test)]
mod test {
    use super::testing::test_captures;

    use indoc::indoc;

    #[test]
    fn test_has_ancestor() {
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
        test_captures(query, input);
    }

    #[test]
    fn test_has_parent() {
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
        test_captures(query, input);
    }
}
