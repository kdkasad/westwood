; extends

(
    (string_content) @injection.content
    (#lua-match? @injection.content "; *tsquery\n")
    (#set! injection.language "query")
)
