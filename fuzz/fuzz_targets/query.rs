#![no_main]
//! Arbitrary XPath against arbitrary XML must never panic.
//!
//! The expression is the more interesting half: it comes from the
//! caller rather than the document, and an application that builds one
//! by concatenating user input is the normal case rather than the
//! exceptional one.
//!
//! The three query functions share a compile step, so this checks they
//! agree about whether an expression is usable at all — one accepting
//! what another rejects would be a contradiction across the API.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };

    // First line is the expression, the rest is the document. Splitting
    // this way lets the fuzzer mutate either half independently.
    let (expression, source) = match text.split_once('\n') {
        Some(pair) => pair,
        None => (text, "<r/>"),
    };

    let Ok(doc) = oxml_wasm::core::parse(source) else {
        return;
    };
    let ns: Vec<String> = Vec::new();

    let text_result = oxml_wasm::core::query_text(&doc, expression, &ns);
    let value_result = oxml_wasm::core::query_value(&doc, expression, &ns);
    let count_result = oxml_wasm::core::query_count(&doc, expression, &ns);

    assert_eq!(
        text_result.is_ok(),
        value_result.is_ok(),
        "query_text and query_value disagree about whether an expression compiles"
    );
    assert_eq!(
        text_result.is_ok(),
        count_result.is_ok(),
        "query_text and query_count disagree about whether an expression compiles"
    );

    // A node-set's count must match the number of texts returned for
    // it. `query_text` falls back to the value for a non-node-set, so
    // only compare when the expression actually selected nodes.
    if let (Ok(texts), Ok(count)) = (&text_result, &count_result) {
        if *count > 0 {
            assert_eq!(
                texts.len(),
                *count,
                "query_text returned {} strings for {} nodes",
                texts.len(),
                count
            );
        }
    }
});
