; extends

(
    (block_comment) @comment
    .
    (raw_string_literal (string_content) @injection.content)
    (#eq? @comment "/* query */")
    (#set! injection.language "query")
)

(
    (block_comment) @comment
    .
    (raw_string_literal (string_content) @injection.content)
    (#eq? @comment "/* c */")
    (#set! injection.language "c")
)
