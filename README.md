# Westwood

The successor to [eastwood](https://github.com/novafacing/eastwood-tidy),
a linter for Purdue's CS 240 course.

Westwood is a linter for C code. It's written in Rust and uses the Tree-sitter
parser library.

## Status

This project is very new and is currently a work in progress.
Development began on March 6, 2025.
If this message hasn't been removed by the end of summer 2025, this project has
probably been abandoned.

## Developing

To make writing Tree-sitter queries and inline C tests easier, integrations for
Neovim have been added. Ensure that you have the `exrc` option enabled in Neovim
and start Neovim in the root of this repository. This will load the `.nvim.lua`
configuration script in this directory, which itself configures Tree-sitter by
loading additional queries to provide syntax highlighting for C code and
Tree-sitter queries inside of string literals in the Rust code.
