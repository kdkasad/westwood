pub mod api;
pub mod rule1a;
pub mod rule1b;
pub mod rule1c;
pub mod rule1d;

use self::api::Rule;

/// Returns a [Vec] of all [rules][Rule].
pub fn get_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(rule1a::Rule1a {}),
        Box::new(rule1b::Rule1b {}),
        Box::new(rule1c::Rule1c {}),
        Box::new(rule1d::Rule1d {}),
    ]
}
