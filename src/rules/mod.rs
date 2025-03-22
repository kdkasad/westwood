// Copyright (C) 2025 Kian Kasad <kian@kasad.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod api;
pub mod rule11a;
pub mod rule1a;
pub mod rule1b;
pub mod rule1c;
pub mod rule1d;
pub mod rule2a;
pub mod rule2b;
pub mod rule3a;
pub mod rule3b;
pub mod rule3c;
pub mod rule3d;
pub mod rule3e;
pub mod rule3f;

use self::api::Rule;

#[must_use]
/// Returns a [Vec] of all [rules][Rule].
pub fn get_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(rule1a::Rule1a {}),
        Box::new(rule1b::Rule1b {}),
        Box::new(rule1c::Rule1c {}),
        Box::new(rule1d::Rule1d {}),
        Box::new(rule2a::Rule2a {}),
        Box::new(rule2b::Rule2b {}),
        Box::new(rule3a::Rule3a {}),
        Box::new(rule3b::Rule3b {}),
        Box::new(rule3c::Rule3c {}),
        Box::new(rule3d::Rule3d {}),
        Box::new(rule3e::Rule3e {}),
        Box::new(rule3f::Rule3f {}),
        Box::new(rule11a::Rule11a::new(Some(3))),
    ]
}
