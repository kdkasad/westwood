use tree_sitter::{Query, QueryCapture, QueryCursor, StreamingIterator as _, Tree};

/// Helper to handle the boilerplate of creating queries and iterating over the captures.
pub struct QueryHelper<'src> {
    query: Query,
    tree: &'src Tree,
    code: &'src [u8],
}

impl<'src> QueryHelper<'src> {
    /// Constructs a new [QueryHelper].
    ///
    /// # Arguments
    ///
    /// - `query_src`: Tree-sitter query to execute.
    /// - `tree`: Tree to execute query on.
    /// - `code`: Source text/code that `tree` represents.
    pub fn new(query_src: &str, tree: &'src Tree, code: &'src [u8]) -> Self {
        let query =
            Query::new(&tree_sitter_c::LANGUAGE.into(), query_src).expect("Failed to parse query");
        Self { query, tree, code }
    }

    /// Executes a callback for each capture obtained by the query.
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
        while let Some(qmatch) = captures.next() {
            let capture = qmatch.0.captures[qmatch.1];
            let name = capture_names[capture.index as usize];
            handler(name, capture);
        }
    }
}
