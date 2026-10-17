// SPDX-License-Identifier: MIT OR Apache-2.0
//! Characterize the bounded converter's node-kind and scalar contracts.

use oxml_wasm::core::{parse, query_text, query_value};

#[test]
fn comments_and_processing_instructions_keep_scalar_semantics() {
    let doc = parse("<r><!--note--><?target data?><child/>text</r>")
        .expect("well-formed");
    for (query, expected) in [
        ("//comment()", "note"),
        ("//processing-instruction()", "data"),
    ] {
        assert_eq!(
            query_value(&doc, query, &[]).expect("scalar value"),
            expected
        );
        assert_eq!(query_text(&doc, query, &[]).expect("direct text"), [""]);
    }
    assert_eq!(query_value(&doc, "/r", &[]).expect("element value"), "text");
    assert_eq!(query_text(&doc, "/", &[]).expect("root text"), ["text"]);
}

#[test]
fn text_and_namespace_nodes_keep_their_direct_values() {
    let doc =
        parse("<r xmlns:m='urn:m'>plain<child/>tail</r>").expect("well-formed");
    assert_eq!(
        query_text(&doc, "/r/text()", &[]).expect("text nodes"),
        ["plain", "tail"]
    );
    assert_eq!(
        query_text(&doc, "/r/namespace::m", &[]).expect("namespace"),
        ["urn:m"]
    );
    assert_eq!(
        query_value(&doc, "/r/namespace::m", &[]).expect("scalar namespace"),
        "urn:m"
    );
}
