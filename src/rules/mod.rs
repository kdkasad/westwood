mod rule1a;
mod rule1c;
pub mod api;

use self::api::Rule;

/// Returns a [Vec] of all [rules][Rule].
pub fn get_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(rule1a::Rule1a {}),
        Box::new(rule1c::Rule1c {}),
    ]
}
