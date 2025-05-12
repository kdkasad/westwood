use std::backtrace::Backtrace;

pub fn main() {
    println!("{}", Backtrace::force_capture());
}
