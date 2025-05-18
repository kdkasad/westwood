use crashlog::cargo_metadata;

#[derive(Default)]
struct CliOptions {
    custom_message: bool,
    replace: bool,
}

pub fn main() {
    // Parse CLI options
    let mut opts = CliOptions::default();
    for arg in std::env::args() {
        match arg.as_str() {
            "--custom" => opts.custom_message = true,
            "--replace" => opts.replace = true,
            arg if arg.starts_with("--seed=") => {
                let seed_str = arg.strip_prefix("--seed=").unwrap();
                let seed = seed_str.parse::<u64>().unwrap();
                fastrand::seed(seed);
            }
            _ => (),
        }
    }

    // Set up crashlog
    if opts.custom_message {
        crashlog::setup!(cargo_metadata!(), opts.replace, "{binary} crashed :(");
    } else {
        crashlog::setup!(cargo_metadata!(), opts.replace);
    }

    // Crash
    panic!("Boo!");
}
