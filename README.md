# Westwood

The successor to [eastwood-tidy], a linter for [Purdue's CS 240 course][cs240].

[eastwood-tidy]: https://github.com/novafacing/eastwood-tidy
[cs240]: https://www.cs.purdue.edu/academic-programs/courses/canonical/cs240.html

Westwood is a linter for C code. It's written in Rust and uses the Tree-sitter
parser library. Westwood is entirely human-made and contains no AI-generated
content.

## Status

This project is very new and is currently a work in progress.
Development began on March 6, 2025.
If this message hasn't been removed by the end of summer 2025, this project has
probably been abandoned.

## For users

Westwood is still under active development, so we do not yet provide releases.
You'll have to follow one of the following methods to get a usable copy of Westwood.

### Download from the CI workflow

Westwood's repository has a GitHub Actions workflow which builds binaries for
many platforms for each Pull Request. These may not reflect the latest version
of Westwood, but are likely the easiest way to obtain a copy.

1. Go to the [Build Westwood binaries][build.yml] action.
2. Select a recent run.
3. Download the artifact matching your platform.

[build.yml]: https://github.com/kdkasad/westwood/actions/workflows/build.yml

### Build from sources (using cargo install)

1. You will need to have Rust and Cargo installed.
2. Run the following command:
   ```
   $ cargo install --git https://github.com/kdkasad/westwood
   ```

To uninstall Westwood if installed with this method, just run:
```
$ cargo uninstall westwood
```

### Build from sources (manually)

1. You'll need to have Rust and Cargo installed.
2. Clone Westwood's sources:
   ```
   $ git clone https://github.com/kdkasad/westwood
   ```
3. Navigate into the repository.
   ```
   $ cd westwood
   ```
4. Build in release mode.
   ```
   $ cargo build -r
   ```
5. The compiled executable will be located at `target/release/westwood`.


## For developers

### Contribution guidelines

1. Be respectful and diligent.
2. Include documentation and unit tests.
3. Do not contribute AI-generated content. AI-generated code is based on real
   humans' works, but does not contain any attribution to the people or
   organizations from which it is derived.

### Tooling and editor integrations

To make writing Tree-sitter queries and inline C tests easier, integrations for
Neovim have been added. Ensure that you have the `exrc` option enabled in Neovim
and start Neovim in the root of this repository. This will load the `.nvim.lua`
configuration script in this directory, which itself configures Tree-sitter by
loading additional queries to provide syntax highlighting for C code and
Tree-sitter queries inside of string literals in the Rust code.


## Copyright, license, and contact

Westwood is written and copyrighted by [Kian Kasad] and
[its contributors].
It is made a available under the terms of the [Apache License 2.0](LICENSE).

If you have any questions about Westwood, please reach out to me (Kian).
I will be happy to ~talk your ear off~ answer them.

[Kian Kasad]: https://github.com/kdkasad
[its contributors]: https://github.com/kdkasad/westwood/graphs/contributors
