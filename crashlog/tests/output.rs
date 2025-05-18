//! Tests the output of Crashlog to make sure it prints what we expect.
//! Runs the `crashlog_test_subject` binary and inspects its output.

use pretty_assertions::assert_eq;

use std::process::{Command, Stdio};

macro_rules! test {
    ($funcname:ident, $args:expr, $expected:expr) => {
        #[test]
        fn $funcname() {
            let output = Command::new(env!("CARGO_BIN_EXE_crashlog_test_subject"))
                .args($args.split_whitespace().chain(std::iter::once("--seed=1")))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .env_remove("RUST_BACKTRACE")
                .output()
                .unwrap();
            assert!(output.stdout.is_empty());
            fastrand::seed(1);
            let path =
                std::env::temp_dir().join(format!("{:08x}.txt", fastrand::u64(0..=u64::MAX)));
            let expected = format!(concat!("{path:.0}", $expected), path = path.display());
            assert_eq!(expected, String::from_utf8(output.stderr).unwrap());
        }
    };
}

test!(
    test_default_append,
    "",
    "
thread 'main' panicked at crashlog/src/bin/crashlog_test_subject.rs:33:5:
Boo!
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---

Uh oh! crashlog crashed.

A crash log was saved at the following path:
{path}

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
{path}

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
thread 'main' panicked at crashlog/src/bin/crashlog_test_subject.rs:33:5:
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
