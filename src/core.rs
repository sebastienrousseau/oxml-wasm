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

/// Bindings written as `PREFIX=URI`, the same spelling `oxml-cli`
/// takes for `--ns`, so the two are learned once.
///
/// From oxml 0.0.4 a prefix in an expression resolves against bindings
/// supplied with the query rather than against the document, and an
/// unbound prefix is an error. Without a way to pass them, `//m:item`
/// would have become unanswerable from JavaScript.
fn parse_bindings(namespaces: &[String]) -> Result<Vec<(&str, &str)>, String> {
    let mut out: Vec<(&str, &str)> = Vec::new();
    for entry in namespaces {
        let Some((prefix, uri)) = entry.split_once('=') else {
            return Err(format!(
                "`{entry}` is not a namespace binding; write PREFIX=URI"
            ));
        };
        if prefix.is_empty() {
            return Err("a namespace binding needs a prefix".to_owned());
        }
        if prefix == "xml" {
            return Err(
                "`xml` is bound by the specification and may not be rebound"
                    .to_owned(),
            );
        }
        // Later bindings win, so a caller can layer defaults.
        out.retain(|(p, _)| *p != prefix);
        out.push((prefix, uri));
    }
    Ok(out)
}

fn compile(
    expression: &str,
    namespaces: &[String],
) -> Result<oxml::XPath, String> {
    let bindings = parse_bindings(namespaces)?;
    oxml::XPath::compile_with_namespaces(expression, &bindings)
        .map_err(|e| e.to_string())
}

/// The text of every matched node.
pub fn query_text(
    doc: &oxml::Document,
    expression: &str,
    namespaces: &[String],
) -> Result<Vec<String>, String> {
    let value = compile(expression, namespaces)?.evaluate(doc);
    Ok(value.nodes().map_or_else(
        || vec![value.to_str(doc)],
        |nodes| nodes.iter().map(|n| doc.text(*n)).collect(),
    ))
}

/// The expression's value as a string.
pub fn query_value(
    doc: &oxml::Document,
    expression: &str,
    namespaces: &[String],
) -> Result<String, String> {
    Ok(compile(expression, namespaces)?.evaluate(doc).to_str(doc))
}

/// How many nodes an expression matches.
pub fn query_count(
    doc: &oxml::Document,
    expression: &str,
    namespaces: &[String],
) -> Result<usize, String> {
    Ok(compile(expression, namespaces)?
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
        let titles = query_text(&doc(), "//title", &[]).expect("valid");
        assert_eq!(titles, ["Dune", "Germinal"]);
    }

    #[test]
    fn query_text_on_a_non_node_set_returns_the_value() {
        // Not an error: JavaScript callers should not have to know
        // which expressions yield node-sets before calling.
        assert_eq!(
            query_text(&doc(), "count(//book)", &[]).expect("valid"),
            ["2"]
        );
    }

    #[test]
    fn query_text_reads_attributes() {
        assert_eq!(
            query_text(&doc(), "//book/@lang", &[]).expect("valid"),
            ["en", "fr"]
        );
    }

    #[test]
    fn query_value_formats_numbers_as_xpath_does() {
        let d = doc();
        assert_eq!(query_value(&d, "count(//book)", &[]).expect("valid"), "2");
        assert_eq!(query_value(&d, "1 div 4", &[]).expect("valid"), "0.25");
        assert_eq!(
            query_value(&d, "//book[1]/@lang", &[]).expect("valid"),
            "en"
        );
        assert_eq!(
            query_value(&d, "count(//book) = 2", &[]).expect("valid"),
            "true"
        );
    }

    #[test]
    fn query_count_counts_nodes_and_nothing_else() {
        let d = doc();
        assert_eq!(query_count(&d, "//book", &[]).expect("valid"), 2);
        assert_eq!(query_count(&d, "//missing", &[]).expect("valid"), 0);
        // A number is not a node-set; zero rather than an error.
        assert_eq!(query_count(&d, "count(//book)", &[]).expect("valid"), 0);
    }

    #[test]
    fn a_malformed_expression_is_an_error_on_every_entry_point() {
        let d = doc();
        for bad in ["//[", "count(", "1 +", "//book[", "@@x"] {
            assert!(query_text(&d, bad, &[]).is_err(), "query_text `{bad}`");
            assert!(query_value(&d, bad, &[]).is_err(), "query_value `{bad}`");
            assert!(query_count(&d, bad, &[]).is_err(), "query_count `{bad}`");
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

/// The README's JavaScript examples, checked against the logic they
/// describe.
///
/// The bindings layer is a one-line map per function, so anything the
/// README claims about behaviour is decidable here without a
/// WebAssembly toolchain. Two of its claims were wrong when written:
/// the error message carries a line and column *before* the library's
/// own byte offset, and `query_text` on an expression that is not a
/// node-set returns a one-element array rather than throwing.
#[cfg(test)]
mod readme {
    use super::*;

    const CATALOGUE: &str = r#"<catalogue>
        <book lang="en"><title>Dune</title><price>9.99</price></book>
        <book lang="fr"><title>Germinal</title><price>7.50</price></book>
    </catalogue>"#;

    #[test]
    fn the_quick_start_returns_what_the_readme_says() {
        let doc = parse(CATALOGUE).expect("well-formed");
        assert_eq!(root_name(&doc).as_deref(), Some("catalogue"));
        assert_eq!(
            query_text(&doc, "//title", &[]).expect("valid"),
            ["Dune", "Germinal"]
        );
        assert_eq!(query_count(&doc, "//book", &[]).expect("valid"), 2);
        assert_eq!(
            query_value(&doc, "sum(//price)", &[]).expect("valid"),
            "17.49"
        );
        assert!(!is_well_formed("<a>"));
    }

    #[test]
    fn query_text_on_a_non_node_set_returns_one_element() {
        // Not an error: the value is converted to a string and wrapped,
        // so a caller iterating the result still gets something useful.
        let doc = parse(CATALOGUE).expect("well-formed");
        assert_eq!(
            query_text(&doc, "count(//book)", &[]).expect("valid"),
            ["2"]
        );
        assert_eq!(query_count(&doc, "count(//book)", &[]).expect("valid"), 0);
    }

    #[test]
    fn a_parse_error_leads_with_line_and_column() {
        let message = parse("<a></b>").expect_err("mismatched tags");
        assert!(
            message.starts_with("1:4: "),
            "the README shows a leading line:column -- got {message:?}"
        );
        assert!(message.contains("</b> closes <a>"), "{message}");
    }

    #[test]
    fn an_invalid_expression_fails_at_query_time_not_parse_time() {
        let doc = parse(CATALOGUE).expect("well-formed");
        assert!(query_value(&doc, "//[", &[]).is_err());
        assert!(query_text(&doc, "//[", &[]).is_err());
        assert!(query_count(&doc, "//[", &[]).is_err());
    }
}

/// Namespace bindings, which arrive as `PREFIX=URI` strings.
#[cfg(test)]
mod bindings {
    use super::{parse, parse_bindings, query_count, query_text};

    const NS: &str =
        r#"<r xmlns:m="urn:u"><m:item>ns</m:item><item>plain</item></r>"#;

    #[test]
    fn a_bound_prefix_selects_only_that_namespace() {
        let doc = parse(NS).expect("well-formed");
        let bound = [String::from("m=urn:u")];
        assert_eq!(
            query_text(&doc, "//m:item", &bound).expect("valid"),
            ["ns"]
        );
    }

    #[test]
    fn only_the_uri_matters_never_the_prefix() {
        // The expression may spell the prefix differently from the
        // document, which is why bindings belong to the query.
        let doc = parse(NS).expect("well-formed");
        let bound = [String::from("q=urn:u")];
        assert_eq!(
            query_text(&doc, "//q:item", &bound).expect("valid"),
            ["ns"]
        );
    }

    #[test]
    fn an_unprefixed_name_matches_no_namespace_only() {
        let doc = parse(NS).expect("well-formed");
        assert_eq!(query_text(&doc, "//item", &[]).expect("valid"), ["plain"]);
    }

    #[test]
    fn an_unbound_prefix_is_an_error() {
        let doc = parse(NS).expect("well-formed");
        assert!(query_count(&doc, "//m:item", &[]).is_err());
    }

    #[test]
    fn later_bindings_win() {
        let entries = [String::from("a=urn:one"), String::from("a=urn:two")];
        let got = parse_bindings(&entries).expect("valid");
        assert_eq!(got, vec![("a", "urn:two")]);
    }

    #[test]
    fn a_malformed_binding_is_rejected() {
        for bad in ["bogus", "=urn:u", "xml=urn:u"] {
            let entries = [bad.to_owned()];
            assert!(
                parse_bindings(&entries).is_err(),
                "{bad} should be rejected"
            );
        }
    }

    #[test]
    fn a_uri_may_contain_an_equals_sign() {
        let entries = [String::from("m=urn:u?a=b")];
        let got = parse_bindings(&entries).expect("valid");
        assert_eq!(got[0].1, "urn:u?a=b");
    }
}
