#![no_main]
//! Arbitrary bytes must never panic the exported operations.
//!
//! These functions sit directly behind the `#[wasm_bindgen]` layer, so
//! a panic here is a WebAssembly trap: the module aborts and every
//! subsequent call on that instance fails, not just the offending one.
//! JavaScript sees an opaque `unreachable` rather than the thrown
//! error the API documents.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = core::str::from_utf8(data) else {
        return;
    };

    // `is_well_formed` must agree with `parse` about what is valid.
    // They are separate entry points, and a document that parses but
    // is reported malformed (or the reverse) is a contradiction a
    // caller cannot work around.
    let parsed = oxml_wasm::core::parse(source);
    assert_eq!(
        parsed.is_ok(),
        oxml_wasm::core::is_well_formed(source),
        "parse and is_well_formed disagree about the same input"
    );

    if let Ok(doc) = parsed {
        let _ = oxml_wasm::core::root_name(&doc);
    }
});
