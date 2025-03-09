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


//! # Rule I:D
//!
//! ```text
//!  D. All global variables must be started with prefix "g_".
//!     Declarations/definitions should be at the top of the file.
//!
//!      Example: int g_temperature = 0;
//!
//!     Global variable use should be avoided unless absolutely necessary.
//! ```
//!
//! # Implementation notes
//!
//! This rule requires that all top-level declarations come before the first function in the file.
//! The code standard just says "Declarations/definitions should be at the top of the file," so
//! I interpret that as meaning all declarations/definitions and not just global variable
//! declarations.

use indoc::indoc;
use tree_sitter::{Point, QueryCapture, Tree};

use crate::{helpers::QueryHelper, rules::api::Rule};

/// Tree-sitter query for Rule I:D.
const QUERY_STR: &'static str = indoc! {
    /* query */
    r#"
    (
        (_ declarator: (identifier) @global.no_g_prefix)
        (#not-match? @global.no_g_prefix "^g_")
        (#not-has-parent? @global.no_g_prefix function_declarator)
        (#not-has-ancestor? @global.no_g_prefix function_definition)
    )

    (translation_unit (function_definition) @function)
    (translation_unit
        [
            (declaration) @declaration.top_level ; variable or function declaration
            (type_specifier) @declaration.top_level ; struct/enum/etc type declaration
            (type_definition) @declaration.top_level ; typedef
        ]
    )
    "#
};

/// # Rule I:D.
///
/// See module-level documentation for details.
pub struct Rule1d {}

impl Rule for Rule1d {
    fn check(&self, filename: &str, tree: &Tree, code: &[u8]) {
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let mut after_function = false;
        helper.for_each_capture(|name: &str, capture: QueryCapture| {
            // For captures that aren't problems, process them as needed and return
            match name {
                "function" => {
                    after_function = true;
                    return;
                }
                "declaration.top_level" if after_function => (),
                "declaration.top_level" => return,
                _ => (),
            }
            let message: &str = match name {
                "global.no_g_prefix" => "Global variables must be prefixed with \"g_\"",
                "declaration.top_level" => {
                    "All top-level declarations must come before function definitions"
                }
                _ => unreachable!(),
            };
            let loc: Point = capture.node.start_position();
            println!("{}:{}:{}: {}", filename, loc.row, loc.column, message);
            println!("{}", capture.node.utf8_text(code).unwrap());
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::testing::test_captures;

    use indoc::indoc;

    use super::QUERY_STR;

    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    #[test]
    fn rule1d() {
        let input = indoc! { /* c */ r#"
            int an_int;
            //!? declaration.top_level
                //!? global.no_g_prefix
            void *a_g_ptr = NULL;
            //!? declaration.top_level
                  //!? global.no_g_prefix

            char *foo() {
            //!? function
                return NULL;
            }

            int g_int = 1;
            //!? declaration.top_level
            char g_char_array[10];
            //!? declaration.top_level
            struct {
            //!? declaration.top_level
                int x;
            } another_global;
              //!? global.no_g_prefix
        "#};
        test_captures(QUERY_STR, input);
    }
}
