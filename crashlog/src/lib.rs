//! # Crashlog: Panic handling for humans
//!
//! Inspired by [human-panic](https://lib.rs/crates/human-panic), but with the following
//! goals/improvements:
//! - Fewer dependencies
//!   - Uses [`std::backtrace`] for backtraces instead of a third-party crate.
//!   - Writes logs in a plain-text format; no need for [`serde`][serde].
//!   - Simplifies color support so third-party libraries aren't needed.
//! - Customizable message
//! - Includes timestamps in logs
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
//! Timestamp: 2025-05-12 22:10:11.191447 UTC
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
//! Simply call [`crashlog::setup!()`][crate::setup!] to register the panic handler.
//!
//! ```ignore
//! crashlog::setup!(ProgramMetadata { /* ... */ }, false);
//! ```
//!
//! You can use the [`cargo_metadata!()`] helper macro to automatically extract the metadata from
//! your `Cargo.toml` file.
//!
//! ```compile_fail
//! // This example doesn't compile because tests/examples don't have the proper metadata
//! // set by Cargo.
//! use crashlog::cargo_metadata;
//! crashlog::setup!(cargo_metadata!().capitalized(), false);
//! ```
//!
//! You can also provide a default placeholder in case some metadata entries are missing, instead
//! of that causing a compilation error.
//!
//! ```
//! # use crashlog::cargo_metadata;
//! crashlog::setup!(cargo_metadata!(default = "(unknown)"), true);
//! ```
//!
//! Finally, you can provide your own panic message to be printed to the user. See [`setup!()`] for
//! information on how to do so.
//!
//! ```
//! # use crashlog::cargo_metadata;
//! crashlog::setup!(cargo_metadata!(default = "(unknown)"), false, "\
//! {package} crashed. Please go to {repository}/issues/new
//! and paste the contents of {log_path}.
//! ");
//! ```
//!
//! # Implementation notes
//!
//! ## When Crashlog fails
//!
//! Creating the crash log file can fail. If it does, the original panic hook is called,
//! regardless of the value of the `replace` argument to [`setup!()`].
//!
//! ## Backtrace formatting
//!
//! The backtrace is handled by [`std::backtrace`], and looks different in debug mode vs. release
//! mode. The backtrace in the example log above is produced by a program compiled in release mode,
//! as that resembles production crashes.
//!
//! Run `cargo run --example backtrace` with and without the `-r` flag in this project's repository
//! to see the difference.

use std::{
    backtrace::Backtrace,
    borrow::Cow,
    fs::File,
    io::{BufWriter, Write},
    panic::PanicHookInfo,
    path::PathBuf,
};

use chrono::{DateTime, Utc};

/// Attempts to generate a crash log and write it to a file.
/// The file is placed in a temporary directory as given by [`std::env::temp_dir()`].
/// If creating or writing to the file fails, `None` is returned, otherwise `Some` is returned with
/// the path of the log file.
///
/// This is an internal function, and should not be called by users of Crashlog.
#[doc(hidden)]
pub fn try_generate_report(
    metadata: &ProgramMetadata,
    info: &PanicHookInfo,
    timestamp: &DateTime<Utc>,
    backtrace: &Backtrace,
) -> Option<PathBuf> {
    // Construct filename
    let mut path = std::env::temp_dir();
    path.push(format!("{:08x}.txt", fastrand::u64(0..=u64::MAX)));

    // Open file and create buffer
    let file = File::create(&path).ok()?;
    let mut w = BufWriter::new(file);

    // Write build information
    let os = os_info::get();
    writeln!(w, "Package: {}", metadata.package).ok()?;
    writeln!(w, "Binary: {}", metadata.binary).ok()?;
    writeln!(w, "Version: {}", metadata.version).ok()?;

    writeln!(w).ok()?;

    // Write system information
    writeln!(w, "Architecture: {}", os.architecture().unwrap_or("(unknown)")).ok()?;
    writeln!(w, "Operating system: {os}").ok()?;
    writeln!(w, "Timestamp: {timestamp}").ok()?;

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
    write!(w, "{backtrace}").ok()?;

    w.flush().ok()?;
    Some(path)
}

/// Wrapper function for macro hygiene
#[doc(hidden)]
pub fn get_timestamp() -> DateTime<Utc> {
    chrono::Utc::now()
}

/// Registers Crashlog's panic handler.
///
/// The first argument is a `metadata` structure which provides information about the program which
/// will be included in the crash log file and in the message printed to the user.
///
/// If the second argument, `replace`, is `false`, Crashlog's panic handler will be appended to the
/// current (or if none is set, the default) panic handler. If `true`, the current panic handler
/// will be replaced.
///
/// The optional third argument allows you to specify a custom message to be printed to the user.
/// This argument must be a string literal. It should use the regular [`std::fmt`] syntax for
/// interpolating values. The fields of the `metadata` structure are all available as
/// [named arguments][1], as well as `log_path`, which represents the path of the crash log file.
/// For example, `"{package} crash log saved at {log_path}"`.
/// If this argument is not given, [`DEFAULT_USER_MESSAGE_TEMPLATE`] is used.
///
/// [1]: std::fmt#named-parameters
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
#[macro_export]
macro_rules! setup {
    ($metadata:expr, $replace:expr) => {
        $crate::setup!(
            $metadata,
            $replace,
            // WARNING: If changing the message below, also change DEFAULT_USER_MESSAGE_TEMPLATE
            "\
Uh oh! {package} crashed.

A crash log was saved at the following path:
{log_path}

To help us figure out why this happened, please report this crash.
Either open a new issue on GitHub [1] or send an email to the author(s) [2].
Attach the file listed above or copy and paste its contents into the report.

[1]: {repository}/issues/new
[2]: {authors}

For your privacy, we don't automatically collect any information, so we rely on
users to submit crash reports to help us find issues. Thank you!"
        )
    };

    ($metadata:expr, $replace:expr, $template:literal) => {{
        let metadata = $metadata;
        let replace = $replace;
        let old_hook = std::panic::take_hook();
        let new_hook = ::std::boxed::Box::new(move |info: &::std::panic::PanicHookInfo| {
            // Get timestamp before running old hook
            let timestamp = $crate::get_timestamp();

            if !replace {
                old_hook(info);
            }

            if let Some(log_path) =
                $crate::try_generate_report(&metadata, info, &timestamp, &::std::backtrace::Backtrace::force_capture())
            {
                if <::std::io::Stderr as ::std::io::IsTerminal>::is_terminal(&::std::io::stderr()) {
                    eprint!("\x1b[31m");
                }
                if !replace {
                    eprintln!("\n---\n");
                }
                eprintln!(
                    // Use all format specifiers with widths of 0 so they don't actually get
                    // produced. This is to silence the unused argument error.
                    concat!("{package:.0}{binary:.0}{version:.0}{repository:.0}{authors:.0}{log_path:.0}", $template),
                    package = metadata.package,
                    binary = metadata.binary,
                    version = metadata.version,
                    repository = metadata.repository,
                    authors = metadata.authors,
                    log_path = log_path.display(),
                );
                if <::std::io::Stderr as ::std::io::IsTerminal>::is_terminal(&::std::io::stderr()) {
                    eprint!("\x1b[m");
                }
            } else if !replace {
                // If creating the crash log failed, and we didn't already run the default hook,
                // run it now.
                old_hook(info);
            }
        });
        ::std::panic::set_hook(new_hook);
    }};
}

/// Default user message template
pub const DEFAULT_USER_MESSAGE_TEMPLATE: &str = "\
Uh oh! {package} crashed.

A crash log was saved at the following path:
{log_path}

To help us figure out why this happened, please report this crash.
Either open a new issue on GitHub [1] or send an email to the author(s) [2].
Attach the file listed above or copy and paste its contents into the report.

[1]: {repository}/issues/new
[2]: {authors}

For your privacy, we don't automatically collect any information, so we rely on
users to submit crash reports to help us find issues. Thank you!";

/// Metadata about the program to be printed in the crash report.
///
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
    /// crashlog::setup!(cargo_metadata!(default = "").capitalized(), false);
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
