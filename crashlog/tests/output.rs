//! Tests the output of Crashlog to make sure it prints what we expect.
//! Runs the `crashlog_test_subject` binary and inspects its output.

use pretty_assertions::assert_eq;

use std::{
    iter::once,
    process::{Command, Stdio},
};

macro_rules! test {
    ($funcname:ident, $args:expr, $expected:expr) => {
        #[test]
        fn $funcname() {
            let output = Command::new(env!("CARGO_BIN_EXE_crashlog_test_subject"))
                .args($args.split_whitespace())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .env_remove("RUST_BACKTRACE")
                .output()
                .unwrap();
            assert!(output.stdout.is_empty());
            assert_eq!($expected, strip_log_path(&output.stderr));
        }
    };
}

// Since the log path is not predictable, we need to strip it
fn strip_log_path(bytes: &[u8]) -> String {
    let text = std::str::from_utf8(bytes).expect("Expected UTF-8");
    text.lines()
        .filter(|line| !line.ends_with(".txt"))
        .flat_map(|line| line.chars().chain(once('\n')))
        .collect()
}

test!(
    test_default_append,
    "",
    "
thread 'main' panicked at crashlog/src/bin/crashlog_test_subject.rs:28:5:
Boo!
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---

Uh oh! crashlog crashed.

A crash log was saved at the following path:

To help us figure out why this happened, please report this crash.
Either open a new issue on GitHub [1] or send an email to the author(s) [2].
Attach the file listed above or copy and paste its contents into the report.

[1]: https://github.com/kdkasad/westwood/issues/new
[2]: Kian Kasad <kian@kasad.com>

For your privacy, we don't automatically collect any information, so we rely on
users to submit crash reports to help us find issues. Thank you!
"
);

test!(
    test_default_replace,
    "--replace",
    "\
Uh oh! crashlog crashed.

A crash log was saved at the following path:

To help us figure out why this happened, please report this crash.
Either open a new issue on GitHub [1] or send an email to the author(s) [2].
Attach the file listed above or copy and paste its contents into the report.

[1]: https://github.com/kdkasad/westwood/issues/new
[2]: Kian Kasad <kian@kasad.com>

For your privacy, we don't automatically collect any information, so we rely on
users to submit crash reports to help us find issues. Thank you!
"
);

test!(
    test_custom_append,
    "--custom",
    "
thread 'main' panicked at crashlog/src/bin/crashlog_test_subject.rs:28:5:
Boo!
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---

crashlog_test_subject crashed :(
"
);

test!(
    test_custom_replace,
    "--custom --replace",
    "\
crashlog_test_subject crashed :(
"
);
