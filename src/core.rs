// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! The binding logic, free of `wasm-bindgen`.
//!
//! Everything the bindings actually *do* lives here, returning `String`
//! errors. `JsError` can only be constructed meaningfully against a
//! JavaScript runtime, so a test that had to build one could not run
//! under `cargo test` — keeping the logic on this side means the
//! behaviour is covered natively and the `#[wasm_bindgen]` layer above
//! is a one-line map per function, with nothing left in it to get wrong.

/// Parse a document, or describe why it is not well-formed.
pub fn parse(source: &str) -> Result<oxml::Document, String> {
    oxml::parse(source).map_err(|e| {
        let (line, column) = e.line_column(source);
        format!("{line}:{column}: {e}")
    })
}

/// The local name of the root element, if there is one.
pub fn root_name(doc: &oxml::Document) -> Option<String> {
    doc.root_element()
        .and_then(|r| doc.element_name(r))
        .map(|n| n.local.clone())
}

fn compile(expression: &str) -> Result<oxml::XPath, String> {
    oxml::XPath::compile(expression).map_err(|e| e.to_string())
}

/// The text of every matched node.
pub fn query_text(
    doc: &oxml::Document,
    expression: &str,
) -> Result<Vec<String>, String> {
    let value = compile(expression)?.evaluate(doc);
    Ok(value.nodes().map_or_else(
        || vec![value.to_str(doc)],
        |nodes| nodes.iter().map(|n| doc.text(*n)).collect(),
    ))
}

/// The expression's value as a string.
pub fn query_value(
    doc: &oxml::Document,
    expression: &str,
) -> Result<String, String> {
    Ok(compile(expression)?.evaluate(doc).to_str(doc))
}

/// How many nodes an expression matches.
pub fn query_count(
    doc: &oxml::Document,
    expression: &str,
) -> Result<usize, String> {
    Ok(compile(expression)?
        .evaluate(doc)
        .nodes()
        .map_or(0, <[oxml::NodeId]>::len))
}

/// Whether a document is well-formed, without keeping it.
pub fn is_well_formed(source: &str) -> bool {
    oxml::parse(source).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "<library count=\"2\">\
        <book lang=\"en\"><title>Dune</title></book>\
        <book lang=\"fr\"><title>Germinal</title></book>\
        </library>";

    fn doc() -> oxml::Document {
        parse(DOC).expect("well-formed")
    }

    #[test]
    fn parse_accepts_a_well_formed_document() {
        assert!(parse(DOC).is_ok());
        assert!(parse("<a/>").is_ok());
        assert!(parse("<?xml version=\"1.0\"?><a>é 😀</a>").is_ok());
    }

    #[test]
    fn a_parse_error_carries_a_position() {
        // In a browser this is the whole diagnostic the user sees, so
        // "failed" without a location would be useless.
        let e = parse("<a><b></a>").expect_err("not well-formed");
        assert!(e.contains(':'), "{e}");
        let (line, rest) = e.split_once(':').expect("line:column");
        assert!(line.parse::<u32>().is_ok(), "{e}");
        assert!(
            rest.split(':')
                .next()
                .and_then(|c| c.trim().parse::<u32>().ok())
                .is_some(),
            "{e}"
        );
    }

    #[test]
    fn root_name_reads_the_document_element() {
        assert_eq!(root_name(&doc()).as_deref(), Some("library"));
    }

    #[test]
    fn query_text_returns_one_entry_per_node() {
        let titles = query_text(&doc(), "//title").expect("valid");
        assert_eq!(titles, ["Dune", "Germinal"]);
    }

    #[test]
    fn query_text_on_a_non_node_set_returns_the_value() {
        // Not an error: JavaScript callers should not have to know
        // which expressions yield node-sets before calling.
        assert_eq!(query_text(&doc(), "count(//book)").expect("valid"), ["2"]);
    }

    #[test]
    fn query_text_reads_attributes() {
        assert_eq!(
            query_text(&doc(), "//book/@lang").expect("valid"),
            ["en", "fr"]
        );
    }

    #[test]
    fn query_value_formats_numbers_as_xpath_does() {
        let d = doc();
        assert_eq!(query_value(&d, "count(//book)").expect("valid"), "2");
        assert_eq!(query_value(&d, "1 div 4").expect("valid"), "0.25");
        assert_eq!(query_value(&d, "//book[1]/@lang").expect("valid"), "en");
        assert_eq!(
            query_value(&d, "count(//book) = 2").expect("valid"),
            "true"
        );
    }

    #[test]
    fn query_count_counts_nodes_and_nothing_else() {
        let d = doc();
        assert_eq!(query_count(&d, "//book").expect("valid"), 2);
        assert_eq!(query_count(&d, "//missing").expect("valid"), 0);
        // A number is not a node-set; zero rather than an error.
        assert_eq!(query_count(&d, "count(//book)").expect("valid"), 0);
    }

    #[test]
    fn a_malformed_expression_is_an_error_on_every_entry_point() {
        let d = doc();
        for bad in ["//[", "count(", "1 +", "//book[", "@@x"] {
            assert!(query_text(&d, bad).is_err(), "query_text `{bad}`");
            assert!(query_value(&d, bad).is_err(), "query_value `{bad}`");
            assert!(query_count(&d, bad).is_err(), "query_count `{bad}`");
        }
    }

    #[test]
    fn is_well_formed_agrees_with_parse() {
        for src in [
            DOC,
            "<a/>",
            "<a><b/></a>",
            "<a><b></a>",
            "<a>",
            "",
            "not xml",
            "<a></a><b></b>",
        ] {
            assert_eq!(
                is_well_formed(src),
                parse(src).is_ok(),
                "disagreement on `{src}`"
            );
        }
    }
}
