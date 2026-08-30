// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! Using the crate from Rust, without a JavaScript runtime.
//!
//! Run with:
//!
//! ```text
//! cargo run --example query_a_document
//! ```
//!
//! `oxml_wasm::core` is the part that decides anything; the
//! `#[wasm_bindgen]` exports above it are a thin translation into
//! types that can cross into JavaScript. Everything here is the same
//! code a browser runs.

use oxml_wasm::core;

fn main() {
    const DOCUMENT: &str = r#"<?xml version="1.0"?>
<catalogue xmlns:m="urn:example:media">
  <book><title>Dune</title><year>1965</year></book>
  <book><title>Solaris</title><year>1961</year></book>
  <m:disc><title>Blade Runner</title></m:disc>
</catalogue>"#;

    assert!(core::is_well_formed(DOCUMENT), "the sample parses");
    assert!(
        !core::is_well_formed("<a><b></a>"),
        "mismatched tags are not well-formed"
    );

    let doc = core::parse(DOCUMENT).expect("the sample is well-formed");
    println!("root: {:?}", core::root_name(&doc));
    assert_eq!(core::root_name(&doc).as_deref(), Some("catalogue"));

    // No prefix in the expression, so no bindings are needed.
    let titles =
        core::query_text(&doc, "//book/title", &[]).expect("valid expression");
    println!("titles: {titles:?}");
    assert_eq!(titles, ["Dune", "Solaris"]);

    let count =
        core::query_count(&doc, "//book", &[]).expect("valid expression");
    println!("books: {count}");
    assert_eq!(count, 2);

    let earliest = core::query_value(&doc, "string(//book[2]/year)", &[])
        .expect("valid expression");
    println!("second book's year: {earliest}");
    assert_eq!(earliest, "1961");

    // A prefix resolves against bindings passed with the query, not
    // against the document -- so it must be supplied here.
    let bindings = vec!["m=urn:example:media".to_owned()];
    let discs = core::query_text(&doc, "//m:disc/title", &bindings)
        .expect("valid expression");
    println!("discs: {discs:?}");
    assert_eq!(discs, ["Blade Runner"]);

    // An unbound prefix is an error, not a silent non-match.
    let unbound = core::query_text(&doc, "//m:disc", &[]);
    println!("unbound prefix: {unbound:?}");
    assert!(unbound.is_err(), "an unbound prefix must be reported");

    // Writing the document back out. The counterpart to `parse`.
    let xml = core::to_xml(&doc);
    println!("serialised {} bytes", xml.len());

    // A fixed point, not a byte-identical copy: a document has more
    // than one valid spelling, so the guarantee is that parsing the
    // output and serialising it again produces the same text.
    let again = core::parse(&xml).expect("the serialised document parses");
    assert_eq!(
        core::to_xml(&again),
        xml,
        "serialisation must be a fixed point"
    );

    // Escaping survives the round trip. `&` in text and `<` inside an
    // attribute both have to come back escaped, or the second parse
    // would see markup where the first saw characters.
    let tricky = core::parse(r#"<a note="x &lt; y">p &amp; q</a>"#)
        .expect("well-formed");
    let out = core::to_xml(&tricky);
    println!("escaped: {out}");
    assert!(out.contains("&amp;"), "text `&` must stay escaped: {out}");
    assert!(
        out.contains("&lt;"),
        "attribute `<` must stay escaped: {out}"
    );
    assert_eq!(
        core::query_value(
            &core::parse(&out).expect("reparses"),
            "string(//a)",
            &[]
        )
        .expect("valid expression"),
        "p & q",
        "the text must survive the round trip unchanged"
    );
}
