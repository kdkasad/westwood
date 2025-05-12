//! # Crashlog: Panic handling for humans
//!
//! Inspired by [human-panic](https://lib.rs/crates/human-panic), but with the following
//! goals/improvements:
//! - Fewer dependencies
//!   - Uses [`std::backtrace`] for backtraces instead of a third-party crate.
//!   - Writes logs in a plain-text format; no need for [`serde`][serde].
//!   - Simplifies color support so third-party libraries aren't needed.
//! - Customizable message (WIP)
//!
//! [serde]: https://crates.io/crates/serde
//!
//! # Example
//!
//! When a program using Crashlog panics, it prints a message like this:
//! ```text
//! $ westwood
//!
//! thread 'main' panicked at src/main.rs:100:5:
//! explicit panic
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//!
//! ---
//!
//! Uh oh! Westwood crashed.
//!
//! A crash log was saved at the following path:
//! /var/folders/sr/kr0r9zfn6wj5pfw35xl47wlm0000gn/T/aaa750e1c7ca7487.txt
//!
//! To help us figure out why this happened, please report this crash.
//! Either open a new issue on GitHub [1] or send an email to the author(s) [2].
//! Attach the file listed above or copy and paste its contents into the report.
//!
//! [1]: https://github.com/kdkasad/westwood/issues/new
//! [2]: Kian Kasad <kian@kasad.com>
//!
//! For your privacy, we don't automatically collect any information, so we rely on
//! users to submit crash reports to help us find issues. Thank you!
//! ```
//!
//! As mentioned in the message, a crash log file is produced, which looks like this:
//! ```text
//! Package: Westwood
//! Binary: westwood
//! Version: 0.0.0
//!
//! Architecture: arm64
//! Operating system: Mac OS 15.4.1 [64-bit]
//!
//! Message: explicit panic
//! Source location: src/main.rs:100
//!
//!    0: std::backtrace::Backtrace::create
//!    1: crashlog::setup::{{closure}}
//!    2: std::panicking::rust_panic_with_hook
//!    3: std::panicking::begin_panic_handler::{{closure}}
//!    4: std::sys::backtrace::__rust_end_short_backtrace
//!    5: _rust_begin_unwind
//!    6: core::panicking::panic_fmt
//!    7: core::panicking::panic_explicit
//!    8: westwood::main::panic_cold_explicit
//!    9: westwood::main
//!   10: std::sys::backtrace::__rust_begin_short_backtrace
//!   11: std::rt::lang_start::{{closure}}
//!   12: std::rt::lang_start_internal
//!   13: _main
//! ```
//!
//! # Usage
//!
//! Simply call [`crashlog::setup()`][crate::setup] with a [`ProgramMetadata`] structure describing
//! your program. The second argument specifies whether to replace to the current panic handler (if
//! `true`) or append to it (if `false`); see [`setup()`] for more details.
//!
//! ```ignore
//! crashlog::setup(ProgramMetadata { /* ... */ }, false);
//! ```
//!
//! You can use the [`cargo_metadata!()`] helper macro to automatically extract the metadata from
//! your `Cargo.toml` file.
//!
//! ```compile_fail
//! // This example doesn't compile because tests/examples don't have the proper metadata
//! // set by Cargo.
//! use crashlog::cargo_metadata;
//! crashlog::setup(cargo_metadata!().capitalized(), false);
//! ```
//!
//! You can also provide a default placeholder in case some metadata entries are missing, instead
//! of that causing a compilation error.
//!
//! ```
//! # use crashlog::cargo_metadata;
//! crashlog::setup(cargo_metadata!(default = "(unknown)"), true);
//! ```

use std::{
    backtrace::Backtrace,
    borrow::Cow,
    fs::File,
    io::{BufWriter, IsTerminal, Write},
    panic::PanicHookInfo,
    path::PathBuf,
};

/// Attempts to generate a crash log and write it to a file.
/// The file is placed in a temporary directory as given by [`std::env::temp_dir()`].
/// If creating or writing to the file fails, `None` is returned, otherwise `Some` is returned with
/// the path of the log file.
fn try_generate_report(metadata: &ProgramMetadata, info: &PanicHookInfo) -> Option<PathBuf> {
    // Construct filename
    let mut path = std::env::temp_dir();
    path.push(format!("{:08x}.txt", getrandom::u64().ok()?));

    // Open file and create buffer
    let file = File::create(&path).ok()?;
    let mut w = BufWriter::new(file);

    // Write build/OS information
    let os = os_info::get();
    writeln!(w, "Package: {}", metadata.package).ok()?;
    writeln!(w, "Binary: {}", metadata.binary).ok()?;
    writeln!(w, "Version: {}", metadata.version).ok()?;
    writeln!(w).ok()?;
    writeln!(w, "Architecture: {}", os.architecture().unwrap_or("(unknown)")).ok()?;
    writeln!(w, "Operating system: {os}").ok()?;

    writeln!(w).ok()?;

    // Write panic cause & location
    let payload_str =
        match (info.payload().downcast_ref::<&str>(), info.payload().downcast_ref::<String>()) {
            (None, None) => "Unknown",
            (Some(str), None) => *str,
            (None, Some(string)) => string.as_str(),
            (Some(_), Some(_)) => unreachable!(),
        };
    writeln!(w, "Message: {payload_str}").ok()?;
    if let Some(loc) = info.location() {
        writeln!(w, "Source location: {}:{}", loc.file(), loc.line()).ok()?;
    } else {
        writeln!(w, "Source location: (unknown)").ok()?;
    }

    writeln!(w).ok()?;

    // Write backtrace
    write!(w, "{}", Backtrace::force_capture()).ok()?;

    w.flush().ok()?;
    Some(path)
}

/// Registers Crashlog's panic handler.
///
/// The `metadata` structure provides information about the program which will be included in the
/// crash log file and in the message printed to the user.
///
/// If `replace` is `false`, Crashlog's panic handler will be appended to the current (or if none is
/// set, the default) panic handler. If `true`, the current panic handler will be replaced.
///
/// With `replace` set to `false`:
/// ```text
/// $ westwood
///
/// thread 'main' panicked at src/main.rs:100:5:
/// explicit panic
/// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
///
///
/// ---
///
/// Uh oh! Westwood crashed.
///
/// A crash log was saved at the following path:
/// /var/folders/sr/kr0r9zfn6wj5pfw35xl47wlm0000gn/T/20ea72fca069a0b7.txt
///
/// To help us figure out why this happened, please report this crash.
/// Either open a new issue on GitHub [1] or send an email to the author(s) [2].
/// Attach the file listed above or copy and paste its contents into the report.
///
/// [1]: https://github.com/kdkasad/westwood/issues/new
/// [2]: Kian Kasad <kian@kasad.com>
///
/// For your privacy, we don't automatically collect any information, so we rely on
/// users to submit crash reports to help us find issues. Thank you!
/// ```
///
/// With `replace` set to `true`:
/// ```text
/// $ westwood
/// Uh oh! Westwood crashed.
///
/// A crash log was saved at the following path:
/// /var/folders/sr/kr0r9zfn6wj5pfw35xl47wlm0000gn/T/20ea72fca069a0b7.txt
///
/// To help us figure out why this happened, please report this crash.
/// Either open a new issue on GitHub [1] or send an email to the author(s) [2].
/// Attach the file listed above or copy and paste its contents into the report.
///
/// [1]: https://github.com/kdkasad/westwood/issues/new
/// [2]: Kian Kasad <kian@kasad.com>
///
/// For your privacy, we don't automatically collect any information, so we rely on
/// users to submit crash reports to help us find issues. Thank you!
/// ```
pub fn setup(metadata: ProgramMetadata, replace: bool) {
    let old_hook = if replace {
        None
    } else {
        Some(std::panic::take_hook())
    };
    let new_hook = Box::new(move |info: &PanicHookInfo| {
        if let Some(hook) = &old_hook {
            hook(info);
        }

        if let Some(report_path) = try_generate_report(&metadata, info) {
            if std::io::stderr().is_terminal() {
                eprint!("\x1b[31m");
            }
            if old_hook.is_some() {
                eprintln!("\n---\n");
            }
            eprintln!(
                "\
Uh oh! {package} crashed.

A crash log was saved at the following path:
{report_path}

To help us figure out why this happened, please report this crash.
Either open a new issue on GitHub [1] or send an email to the author(s) [2].
Attach the file listed above or copy and paste its contents into the report.

[1]: {repo_url}/issues/new
[2]: {authors}

For your privacy, we don't automatically collect any information, so we rely on
users to submit crash reports to help us find issues. Thank you!",
                package = metadata.package,
                report_path = report_path.display(),
                repo_url = metadata.repository,
                authors = metadata.authors,
            );
            if std::io::stderr().is_terminal() {
                eprint!("\x1b[m");
            }
        } else {
            todo!()
        }
    });
    std::panic::set_hook(new_hook);
}

/// Metadata about the program to be printed in the crash report.
/// Typically sourced from `Cargo.toml` using the `CARGO_PKG_*` environment variables.
/// Use [`cargo_metadata!()`] to create a `ProgramMetadata` filled with values from `Cargo.toml`.
#[derive(Debug, Clone)]
pub struct ProgramMetadata {
    pub package: Cow<'static, str>,
    pub binary: Cow<'static, str>,
    pub version: Cow<'static, str>,
    pub repository: Cow<'static, str>,
    pub authors: Cow<'static, str>,
}

impl ProgramMetadata {
    /// Capitalizes the first letter of the package name.
    ///
    /// # Example
    ///
    /// ```
    /// use crashlog::cargo_metadata;
    /// crashlog::setup(cargo_metadata!(default = "").capitalized(), false);
    /// ```
    pub fn capitalized(self) -> Self {
        let mut new = self;
        let mut chars = new.package.chars();
        new.package = chars
            .next()
            .iter()
            .flat_map(|first_letter| first_letter.to_uppercase())
            .chain(chars)
            .collect::<String>()
            .into();
        new
    }
}

/// Macro to generate a [`ProgramMetadata`] structure using information from Cargo.
///
/// The metadata is retrieved from [environment variables set by Cargo][1]. If any of the
/// expected environment variables are not set, compilation will fail. To avoid this, you can
/// provide a default placeholder by providing `default = "..."` as arguments to the macro. The
/// string literal can contain any value.
///
/// [1]: https://doc.rust-lang.org/stable/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
///
/// # Examples
///
/// ```compile_fail
/// # use crashlog::cargo_metadata;
/// // This will fail to compile because examples don't have CARGO_BIN_NAME set.
/// let m = cargo_metadata!();
/// ```
///
/// ```
/// # use crashlog::cargo_metadata;
/// let m = cargo_metadata!(default = "");
/// assert_eq!(m.package, "crashlog");
/// assert_eq!(m.binary, "");
/// ```
#[macro_export]
macro_rules! cargo_metadata {
    () => {
        $crate::ProgramMetadata {
            package: ::std::borrow::Cow::Borrowed(env!("CARGO_PKG_NAME")),
            binary: ::std::borrow::Cow::Borrowed(env!("CARGO_BIN_NAME")),
            version: ::std::borrow::Cow::Borrowed(env!("CARGO_PKG_VERSION")),
            repository: ::std::borrow::Cow::Borrowed(env!("CARGO_PKG_REPOSITORY")),
            authors: $crate::cow_replace(env!("CARGO_PKG_AUTHORS"), ":", ", "),
        }
    };

    (default = $placeholder:literal) => {
        $crate::ProgramMetadata {
            package: ::std::borrow::Cow::Borrowed(
                option_env!("CARGO_PKG_NAME").unwrap_or($placeholder),
            ),
            binary: ::std::borrow::Cow::Borrowed(
                option_env!("CARGO_BIN_NAME").unwrap_or($placeholder),
            ),
            version: ::std::borrow::Cow::Borrowed(
                option_env!("CARGO_PKG_VERSION").unwrap_or($placeholder),
            ),
            repository: ::std::borrow::Cow::Borrowed(
                option_env!("CARGO_PKG_REPOSITORY").unwrap_or($placeholder),
            ),
            authors: option_env!("CARGO_PKG_AUTHORS")
                .map_or(::std::borrow::Cow::Borrowed($placeholder), |s| {
                    $crate::cow_replace(s, ":", ", ")
                }),
        }
    };
}

/// Like [`str::replace()`], but only clones the string if a replacement is needed.
///
/// This is an internal helper function and is not part of Crashlog's API.
/// You should not use this function.
#[doc(hidden)]
pub fn cow_replace<'a>(s: &'a str, from: &str, to: &str) -> Cow<'a, str> {
    if s.contains(from) {
        Cow::Owned(s.replace(from, to))
    } else {
        Cow::Borrowed(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalize_package_name() {
        let metadata = ProgramMetadata {
            package: "crashlog".into(),
            binary: "".into(),
            version: "".into(),
            repository: "".into(),
            authors: "".into(),
        };
        let new = metadata.capitalized();
        assert_eq!("Crashlog", new.package);
        let mut empty = new;
        empty.package = "".into();
        let new = empty.capitalized();
        assert_eq!("", new.package);
    }

    #[test]
    fn metadata_placeholder() {
        // For unit tests, Cargo does not set `CARGO_BIN_NAME`, so we expect the placeholder.
        let metadata = cargo_metadata!(default = "place");
        assert_eq!(metadata.binary, "place");
    }

    #[test]
    fn test_cow_replace() {
        // When found, string should be copied and replaced
        let s = "abc:def";
        assert!(matches!(cow_replace(s, ":", ","), Cow::Owned(val) if val == "abc,def"));

        // When not found, string should not be copied
        let s = "abc:def";
        assert!(matches!(cow_replace(s, "+", " "), Cow::Borrowed(val) if val == s));
    }
}
