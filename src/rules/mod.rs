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
pub mod rule01a;
pub mod rule01b;
pub mod rule01c;
pub mod rule01d;
pub mod rule02a;
pub mod rule02b;
pub mod rule03a;
pub mod rule03b;
pub mod rule03c;
pub mod rule03d;
pub mod rule03e;
pub mod rule03f;
pub mod rule11a;

use self::api::Rule;

#[must_use]
/// Returns a [Vec] of all [rules][Rule].
pub fn get_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(rule01a::Rule01a {}),
        Box::new(rule01b::Rule01b {}),
        Box::new(rule01c::Rule01c {}),
        Box::new(rule01d::Rule01d {}),
        Box::new(rule02a::Rule02a {}),
        Box::new(rule02b::Rule02b {}),
        Box::new(rule03a::Rule03a {}),
        Box::new(rule03b::Rule03b {}),
        Box::new(rule03c::Rule03c {}),
        Box::new(rule03d::Rule03d {}),
        Box::new(rule03e::Rule03e {}),
        Box::new(rule03f::Rule03f {}),
        Box::new(rule11a::Rule11a::new(Some(3))),
    ]
}
