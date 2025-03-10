# Westwood

The successor to [eastwood], a linter for [Purdue's CS 240 course][cs240].

[eastwood]: https://github.com/novafacing/eastwood-tidy
[cs240]: https://www.cs.purdue.edu/academic-programs/courses/canonical/cs240.html

Westwood is a linter for C code. It's written in Rust and uses the Tree-sitter
parser library. Westwood is entirely human-made and contains no AI-generated
content.

## Status

This project is very new and is currently a work in progress.
Development began on March 6, 2025.
If this message hasn't been removed by the end of summer 2025, this project has
probably been abandoned.

## For developers

To make writing Tree-sitter queries and inline C tests easier, integrations for
Neovim have been added. Ensure that you have the `exrc` option enabled in Neovim
and start Neovim in the root of this repository. This will load the `.nvim.lua`
configuration script in this directory, which itself configures Tree-sitter by
loading additional queries to provide syntax highlighting for C code and
Tree-sitter queries inside of string literals in the Rust code.

## Copyright, license, and contact

Westwood is written and copyrighted by [Kian Kasad][kdkasad].
It is made a available under the terms of the [Apache License 2.0](LICENSE).

If you have any questions about Westwood, please reach out to me (Kian).
I will be happy to ~talk your ear off~ answer them.

[kdkasad]: https://github.com/kdkasad
