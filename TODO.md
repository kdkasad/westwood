# To-do list

## Features

- [ ] Implement the rest of the code standard rules (except indentation)
- [ ] Implement indentation checks (II:A)
- [ ] Add option to sort diagnostics by location vs. by rule.
- [ ] Additional output formats:
    - [x] Machine-parseable format (for editors to integrate with)
      - [ ] Figure out a good way to preserve labels/supplementary messages
      - [ ] Come up with a Vim `'errorformat'` string to match the output format
    - [ ] JSON
    - [ ] Output source code annotated with errors in-line
- [ ] Overhaul documentation
- [ ] Figure out and possibly provide configurations for editor integration
  - [ ] Vim
  - [ ] Neovim
  - [ ] VS Code (probably too difficult to be worth it)
- [ ] LSP support? (Probably a massive undertaking)

## Chores

- [ ] Pull out some of the UTF-8 conversions so we do them only once.
- [ ] Unit tests which check the diagnostics being created, not just the query captures.
  - [ ] Make a nice framework for this.
  - [ ] Add diagnostic unit tests to every rule.
