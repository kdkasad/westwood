use crashlog::cargo_metadata;

pub fn main() {
    crashlog::setup!(cargo_metadata!(default = ""), true, "{package} crashed :(");
    panic!();
}
