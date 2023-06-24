// Copyright 2023 Shafish Labs.
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

use llmchain_common::chat_tokens;

#[test]
fn test_token() {
    let input = "This is a sentence   with spaces, hahhahah haha ha";
    let output = chat_tokens(input).unwrap();
    assert_eq!(output, [
        "This",
        " is",
        " a",
        " sentence",
        " ",
        " ",
        " with",
        " spaces",
        ",",
        " ha",
        "hh",
        "ahah",
        " haha",
        " ha"
    ]);
}