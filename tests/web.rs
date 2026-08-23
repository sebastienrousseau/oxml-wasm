// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! The JavaScript-facing surface, exercised in a real runtime.
//!
//! `src/core.rs` is unit-tested natively; what these add is the
//! `#[wasm_bindgen]` layer above it, which cannot run under `cargo
//! test` because it produces and consumes JavaScript values. Run with
//! `wasm-pack test --node`.
//!
//! The methods are called by their Rust names here; `js_name` renames
//! only the surface JavaScript sees.

#![cfg(target_arch = "wasm32")]

use oxml_wasm::{is_well_formed, parse};
use wasm_bindgen_test::wasm_bindgen_test;

const DOC: &str = "<library count=\"2\">\
    <book lang=\"en\"><title>Dune</title></book>\
    <book lang=\"fr\"><title>Germinal</title></book>\
    </library>";

#[wasm_bindgen_test]
fn parse_returns_a_usable_document() {
    let doc = parse(DOC).expect("well-formed");
    assert_eq!(doc.root_name().as_deref(), Some("library"));
    assert!(doc.size() > 0);
}

#[wasm_bindgen_test]
fn parse_rejects_a_malformed_document() {
    // The error must cross the boundary as a JS exception rather than
    // unwinding into a poisoned module.
    assert!(parse("<a><b></a>").is_err());
}

#[wasm_bindgen_test]
fn query_text_crosses_the_boundary_as_an_array() {
    let doc = parse(DOC).expect("well-formed");
    let titles = doc.query_text("//title").expect("valid expression");
    assert_eq!(titles, vec!["Dune".to_owned(), "Germinal".to_owned()]);
}

#[wasm_bindgen_test]
fn query_text_reads_attributes() {
    let doc = parse(DOC).expect("well-formed");
    let langs = doc.query_text("//book/@lang").expect("valid expression");
    assert_eq!(langs, vec!["en".to_owned(), "fr".to_owned()]);
}

#[wasm_bindgen_test]
fn query_value_and_count_agree_with_the_document() {
    let doc = parse(DOC).expect("well-formed");
    assert_eq!(doc.query_value("count(//book)").expect("valid"), "2");
    assert_eq!(doc.query_count("//book").expect("valid"), 2);
    assert_eq!(doc.query_count("//missing").expect("valid"), 0);
}

#[wasm_bindgen_test]
fn a_malformed_expression_is_an_error_on_every_entry_point() {
    let doc = parse(DOC).expect("well-formed");
    assert!(doc.query_text("//[").is_err());
    assert!(doc.query_value("//[").is_err());
    assert!(doc.query_count("//[").is_err());
}

#[wasm_bindgen_test]
fn the_document_survives_a_failed_query() {
    // A thrown error must not leave the instance unusable.
    let doc = parse(DOC).expect("well-formed");
    assert!(doc.query_text("//[").is_err());
    assert_eq!(doc.query_text("//title").expect("valid").len(), 2);
}

#[wasm_bindgen_test]
fn is_well_formed_answers_without_keeping_the_document() {
    assert!(is_well_formed(DOC));
    assert!(!is_well_formed("<a><b></a>"));
    assert!(!is_well_formed(""));
}

#[wasm_bindgen_test]
fn non_ascii_content_survives_the_boundary() {
    let doc = parse("<a><b>Germinal 😀 é 中</b></a>").expect("well-formed");
    let text = doc.query_text("//b").expect("valid");
    assert_eq!(text, vec!["Germinal 😀 é 中".to_owned()]);
}
