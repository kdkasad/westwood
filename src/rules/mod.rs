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
