# Crashlog: Panic handling for humans

Inspired by [human-panic](https://lib.rs/crates/human-panic), but with the following
goals/improvements:
- Fewer dependencies
  - Uses [`std::backtrace`] for backtraces instead of a third-party crate.
  - Writes logs in a plain-text format; no need for [`serde`][serde].
  - Simplifies color support so third-party libraries aren't needed.
- Customizable message (WIP)

[serde]: https://crates.io/crates/serde

## Example

When a program using Crashlog panics, it prints a message like this:
```text
$ westwood

thread 'main' panicked at src/main.rs:100:5:
explicit panic
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---

Uh oh! Westwood crashed.

A crash log was saved at the following path:
/var/folders/sr/kr0r9zfn6wj5pfw35xl47wlm0000gn/T/aaa750e1c7ca7487.txt

To help us figure out why this happened, please report this crash.
Either open a new issue on GitHub [1] or send an email to the author(s) [2].
Attach the file listed above or copy and paste its contents into the report.

[1]: https://github.com/kdkasad/westwood/issues/new
[2]: Kian Kasad <kian@kasad.com>

For your privacy, we don't automatically collect any information, so we rely on
users to submit crash reports to help us find issues. Thank you!
```

As mentioned in the message, a crash log file is produced, which looks like this:
```text
Package: Westwood
Binary: westwood
Version: 0.0.0

Architecture: arm64
Operating system: Mac OS 15.4.1 [64-bit]

Message: explicit panic
Source location: src/main.rs:100

   0: std::backtrace::Backtrace::create
   1: crashlog::setup::{{closure}}
   2: std::panicking::rust_panic_with_hook
   3: std::panicking::begin_panic_handler::{{closure}}
   4: std::sys::backtrace::__rust_end_short_backtrace
   5: _rust_begin_unwind
   6: core::panicking::panic_fmt
   7: core::panicking::panic_explicit
   8: westwood::main::panic_cold_explicit
   9: westwood::main
  10: std::sys::backtrace::__rust_begin_short_backtrace
  11: std::rt::lang_start::{{closure}}
  12: std::rt::lang_start_internal
  13: _main
```

## Usage

Simply call [`crashlog::setup()`][crate::setup] with a [`ProgramMetadata`] structure describing
your program. The second argument specifies whether to replace to the current panic handler (if
`true`) or append to it (if `false`); see [`setup()`] for more details.

```rust
crashlog::setup(ProgramMetadata { /* ... */ }, false);
```

You can use the [`cargo_metadata!()`] helper macro to automatically extract the metadata from
your `Cargo.toml` file.

```rust
// This example doesn't compile because tests/examples don't have the proper metadata
// set by Cargo.
use crashlog::cargo_metadata;
crashlog::setup(cargo_metadata!().capitalized(), false);
```

You can also provide a default placeholder in case some metadata entries are missing, instead
of that causing a compilation error.

```rust
crashlog::setup(cargo_metadata!(default = "(unknown)"), true);
```
