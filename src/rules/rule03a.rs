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

//! # Rule III:A
//!
//! ```text
//!    A. One space must be placed after all structure control, and flow
//!       commands. One space must also be present between the closing
//!       parenthesis and opening brace.
//!
//!       Example: if (temperature == room_temperature) {
//!
//!       Example: while (temperature < room_temperature) {
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use indoc::indoc;
use tree_sitter::Node;

use crate::{helpers::QueryHelper, rules::api::Rule};

use crate::rules::api::SourceInfo;

use super::api::RuleDescription;

/// Tree-sitter query for Rule III:A.
const QUERY_STR: &str = indoc! {
    /* query */
    r#"
    ; Capture control flow statements which have a right parenthesis followed by a brace

    (if_statement
        .
        "if" @keyword
        .
        condition: (parenthesized_expression . "(" @lparen ")" @rparen .)
        consequence: (compound_statement . "{" @lbrace))

    (for_statement
        .
        "for" @keyword
        .
        "(" @lparen
        ")" @rparen
        body: (compound_statement . "{" @lbrace)
        .)

    (while_statement
        "while" @keyword
        .
        condition: (parenthesized_expression . "(" @lparen ")" @rparen .)
        body: (compound_statement . "{" @lbrace))

    (switch_statement
        .
        "switch" @keyword
        .
        condition: (parenthesized_expression . "(" @lparen ")" @rparen .)
        body: (compound_statement . "{" @lbrace))

    ; Note: These last 2 patterns are handled differently in the logic below.

    (do_statement
        .
        "do" @keyword
        .
        body: (compound_statement . "{" @lparen))

    (do_statement
        body: (_)
        .
        "while" @keyword
        .
        condition: (parenthesized_expression . "(" @lparen))
    "#
};

/// # Rule III:A.
///
/// See module-level documentation for details.
pub struct Rule03a {}

impl Rule for Rule03a {
    fn describe(&self) -> &'static RuleDescription {
        &RuleDescription {
            group_number: 3,
            letter: 'A',
            code: "III:A",
            name: "FlowControlSpacing",
            description: "one space must be placed between flow control constructs",
        }
    }

    fn check(&self, SourceInfo { tree, code, .. }: &SourceInfo) -> Vec<Diagnostic<()>> {
        let mut diagnostics = Vec::new();

        // Part 1: Space between parentheses and braces
        let helper = QueryHelper::new(QUERY_STR, tree, code);
        let keyword_capture_i = helper.expect_index_for_capture("keyword");
        let lparen_capture_i = helper.expect_index_for_capture("lparen");
        let rparen_capture_i = helper.expect_index_for_capture("rparen");
        let lbrace_capture_i = helper.expect_index_for_capture("lbrace");
        helper.for_each_match(|qmatch| {
            // The last pattern checks for "do" statements, which do not have parentheses followed
            // by braces, so we skip this check if it's the last pattern.
            // TODO: Avoid using indices to figure this out.
            if qmatch.captures.len() == 4 {
                // Check spacing between ) and {
                let rparen = helper.expect_node_for_capture_index(qmatch, rparen_capture_i);
                let lbrace = helper.expect_node_for_capture_index(qmatch, lbrace_capture_i);
                let message =
                    "Expected a single space between the closing parenthesis and the opening brace";
                if let Some(diagnostic) = check_single_space_between(rparen, lbrace, code, message)
                {
                    diagnostics.push(diagnostic);
                }
            } else {
                // Just in case
                assert_eq!(2, qmatch.captures.len(), "Expected either 2 or 4 captures");
            }

            // Check spacing between keyword and (
            let keyword = helper.expect_node_for_capture_index(qmatch, keyword_capture_i);
            let lparen = helper.expect_node_for_capture_index(qmatch, lparen_capture_i);
            let message =
                format!("Expected a single space after `{}'", &code[keyword.byte_range()]);
            if let Some(diagnostic) = check_single_space_between(keyword, lparen, code, &message) {
                diagnostics.push(diagnostic);
            }
        });

        diagnostics
    }
}

/// Returns a [Diagnostic] if there is not a single space separating the left and right nodes.
/// The returned diagnostic will have a message of `message`.
fn check_single_space_between(
    left: Node,
    right: Node,
    code: &str,
    message: &str,
) -> Option<Diagnostic<()>> {
    if (left.end_byte() + 1) == right.start_byte() {
        // One byte in between
        if code.as_bytes()[left.end_byte()] == b' ' {
            // Valid
            return None;
        }
    }
    Some(
        Diagnostic::warning()
            .with_code("III:A")
            .with_message(message.to_owned())
            .with_label(Label::primary((), left.start_byte()..right.end_byte())),
    )
}

#[cfg(test)]
mod tests {
    // TODO: Test the actual lints produced, because not all of the logic for this rule is
    // encapsulated in the query.

    use std::process::ExitCode;

    use indoc::indoc;

    use crate::helpers::testing::test_captures;

    use super::QUERY_STR;

    #[test]
    fn rule03a_captures() -> ExitCode {
        let input = indoc! {
            /* c */
            r"
            int main() {
                switch (x) {
                //!? keyword
                       //!? lparen
                         //!? rparen
                           //!? lbrace
                }
                switch
                //!? keyword
                (x)
                //!? lparen
                  //!? rparen
                {
                //!? lbrace
                }
                switch(x){
                //!? keyword
                      //!? lparen
                        //!? rparen
                         //!? lbrace
                }

                if (x) {
                //!? keyword
                   //!? lparen
                     //!? rparen
                       //!? lbrace
                }
                if
                //!? keyword
                (x)
                //!? lparen
                  //!? rparen
                {
                //!? lbrace
                }
                if(x){
                //!? keyword
                  //!? lparen
                    //!? rparen
                     //!? lbrace
                }
                if (x) {
                //!? keyword
                   //!? lparen
                     //!? rparen
                       //!? lbrace
                } else if (y) {
                       //!? keyword
                          //!? lparen
                            //!? rparen
                              //!? lbrace
                }
                if(x){
                //!? keyword
                  //!? lparen
                    //!? rparen
                     //!? lbrace
                } else if(y){
                       //!? keyword
                         //!? lparen
                           //!? rparen
                            //!? lbrace
                }

                for (x;x;x) {
                //!? keyword
                    //!? lparen
                          //!? rparen
                            //!? lbrace
                }
                for
                //!? keyword
                (x;x;x)
                //!? lparen
                      //!? rparen
                {
                //!? lbrace
                }
                for(x;x;x){
                //!? keyword
                   //!? lparen
                         //!? rparen
                          //!? lbrace
                }
                for(x;
                //!? keyword
                   //!? lparen
                    x;
                    x){
                     //!? rparen
                      //!? lbrace
                }

                while (x) {
                //!? keyword
                      //!? lparen
                        //!? rparen
                          //!? lbrace
                }
                while
                //!? keyword
                (x)
                //!? lparen
                  //!? rparen
                {
                //!? lbrace
                }
                while(x){
                //!? keyword
                     //!? lparen
                       //!? rparen
                        //!? lbrace
                }

                do {
                //!? keyword
                   //!? lparen
                } while (0);
                  //!? keyword
                        //!? lparen
                do{
                //!? keyword
                  //!? lparen
                }while(0);
                 //!? keyword
                      //!? lparen
            }
            "
        };
        test_captures(QUERY_STR, input)
    }
}
