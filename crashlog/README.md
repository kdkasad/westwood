<!-- cargo-rdme start -->

# Crashlog: Panic handling for humans

Inspired by [human-panic](https://lib.rs/crates/human-panic), but with the following
goals/improvements:
- Fewer dependencies
  - Uses [`std::backtrace`] for backtraces instead of a third-party crate.
  - Writes logs in a plain-text format; no need for [`serde`][serde].
  - Simplifies color support so third-party libraries aren't needed.
- Customizable message
- Includes timestamps in logs

[serde]: https://crates.io/crates/serde

# Example

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
Timestamp: 2025-05-12 22:10:11.191447 UTC

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

# Usage

Simply call [`crashlog::setup!()`][crate::setup!] to register the panic handler.

```rust
crashlog::setup!(ProgramMetadata { /* ... */ }, false);
```

You can use the [`cargo_metadata!()`] helper macro to automatically extract the metadata from
your `Cargo.toml` file.

```rust
// This example doesn't compile because tests/examples don't have the proper metadata
// set by Cargo.
use crashlog::cargo_metadata;
crashlog::setup!(cargo_metadata!().capitalized(), false);
```

You can also provide a default placeholder in case some metadata entries are missing, instead
of that causing a compilation error.

```rust
crashlog::setup!(cargo_metadata!(default = "(unknown)"), true);
```

Finally, you can provide your own panic message to be printed to the user. See [`setup!()`] for
information on how to do so.

```rust
crashlog::setup!(cargo_metadata!(default = "(unknown)"), false, "\
{package} crashed. Please go to {repository}/issues/new
and paste the contents of {log_path}.
");
```

# Implementation notes

## When Crashlog fails

Creating the crash log file can fail. If it does, the original panic hook is called,
regardless of the value of the `replace` argument to [`setup!()`].

## Backtrace formatting

The backtrace is handled by [`std::backtrace`], and looks different in debug mode vs. release
mode. The backtrace in the example log above is produced by a program compiled in release mode,
as that resembles production crashes.

Run `cargo run --example backtrace` with and without the `-r` flag in this project's repository
to see the difference.

<!-- cargo-rdme end -->
