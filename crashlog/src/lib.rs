//! # Human-friendly crash reporting
//!
//! Inspired by [human-panic](https://lib.rs/crates/human-panic).
//!
//! This module provides functionality for producing crash logs on panic and printing a message to
//! the user informing them of the crash and how to report it.

use indoc::indoc;
use std::{
    backtrace::Backtrace,
    borrow::Cow,
    fs::File,
    io::{BufWriter, IsTerminal, Write},
    panic::PanicHookInfo,
    path::PathBuf,
};

type PanicHookHandler = Box<dyn Fn(&PanicHookInfo) + Send + Sync + 'static>;

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

/// Creates and returns the panic hook closure.
fn create_panic_hook(metadata: ProgramMetadata) -> PanicHookHandler {
    Box::new(move |info: &PanicHookInfo| {
        if let Some(report_path) = try_generate_report(&metadata, info) {
            if std::io::stderr().is_terminal() {
                eprintln!("\x1b[31m");
            }
            eprintln!(
                indoc! { "
                    Uh oh! {package} crashed.

                    A crash log was saved at the following path:
                    {report_path}

                    To help us figure out why this happened, please report this crash.
                    Either open a new issue on GitHub [1] or send an email to the author(s) [2].
                    Attach the file listed above or copy and paste its contents into the report.

                    [1]: {repo_url}/issues/new
                    [2]: {authors}

                    For your privacy, we don't automatically collect any information, so we rely on
                    users to submit crash reports to help us find issues. Thank you!" },
                package = metadata.package,
                report_path = report_path.display(),
                repo_url = metadata.repository,
                authors = metadata.authors,
            );
            if std::io::stderr().is_terminal() {
                eprintln!("\x1b[m");
            }
        } else {
            todo!()
        }
    })
}

/// Registers the custom user-friendly panic handler.
pub fn setup(metadata: ProgramMetadata) {
    std::panic::set_hook(create_panic_hook(metadata));
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

#[macro_export]
macro_rules! cargo_metadata {
    () => {
        $crate::ProgramMetadata {
            package: ::std::borrow::Cow::from(env!("CARGO_PKG_NAME")),
            binary: ::std::borrow::Cow::from(env!("CARGO_BIN_NAME")),
            version: ::std::borrow::Cow::from(env!("CARGO_PKG_VERSION")),
            repository: ::std::borrow::Cow::from(env!("CARGO_PKG_REPOSITORY")),
            authors: ::std::borrow::Cow::from(env!("CARGO_PKG_AUTHORS").replace(':', ", ")),
        }
    };
}
