// SPDX-License-Identifier: MIT OR Apache-2.0
//! Public bindings must retain the core's untrusted-input budgets.

#[test]
fn every_query_entry_point_bounds_materialized_strings() {
    let source = format!(
        "<r>{}<n/>{}</r>",
        "x".repeat(512 * 1024),
        "x".repeat(512 * 1024 + 1)
    );
    let doc = oxml_wasm::core::parse(&source).expect("within document budget");
    let results = [
        oxml_wasm::core::query_text(&doc, "string(/r)", &[]).map(|_| ()),
        oxml_wasm::core::query_value(&doc, "string(/r)", &[]).map(|_| ()),
        oxml_wasm::core::query_count(&doc, "string(/r)", &[]).map(|_| ()),
    ];
    for result in results {
        assert!(result.is_err(), "untrusted query ignored the string budget");
    }
}

#[test]
fn parsing_and_well_formedness_refuse_oversized_documents() {
    let source = format!("<r>{}</r>", "x".repeat(8 * 1024 * 1024));
    assert!(oxml_wasm::core::parse(&source).is_err());
    assert!(!oxml_wasm::core::is_well_formed(&source));
}

#[test]
fn node_set_conversion_cannot_bypass_string_budgets() {
    let source = format!("<r>{}<n>tail</n></r>", "x".repeat(1024 * 1024));
    let doc = oxml_wasm::core::parse(&source).expect("strict source profile");
    assert!(oxml_wasm::core::query_text(&doc, "/r | /r/n", &[]).is_err());
    assert!(oxml_wasm::core::query_value(&doc, "/r", &[]).is_err());
    assert_eq!(oxml_wasm::core::query_count(&doc, "/r", &[]), Ok(1));
}
