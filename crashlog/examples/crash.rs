use crashlog::cargo_metadata;

/// Immediately panic
pub fn main() {
    crashlog::setup(cargo_metadata!(), false);
    panic!();
}
