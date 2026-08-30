// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! WebAssembly bindings for [`oxml`].
//!
//! Being pure Rust with no C dependency is what makes this possible at
//! all: `libxml`-based crates cannot target WebAssembly without a
//! libxml2 toolchain, which is most of the reason a browser-side `XPath`
//! has not been practical in Rust.
//!
//! The surface is deliberately small. JavaScript already has
//! `DOMParser`; what it does not have is a fast, dependency-free `XPath`
//! that behaves identically to the server-side one. That is what these
//! bindings expose.

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

/// The work, without the `wasm-bindgen` layer.
///
/// Public so it can be benchmarked and so the crate is usable from
/// Rust as well as from JavaScript. The exports below are a thin
/// translation of these into types `wasm-bindgen` can carry across
/// the boundary; everything that decides anything lives here.
pub mod core;

/// A parsed XML document.
#[wasm_bindgen]
#[derive(Debug)]
pub struct Document {
    inner: oxml::Document,
}

/// Parse an XML document.
///
/// # Errors
///
/// Returns a `JsError` carrying the position and reason if the input
/// is not well-formed.
#[wasm_bindgen]
pub fn parse(source: &str) -> Result<Document, JsError> {
    core::parse(source)
        .map(|inner| Document { inner })
        .map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
impl Document {
    /// The number of nodes, including the document root.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn size(&self) -> usize {
        self.inner.len()
    }

    /// The name of the root element, if there is one.
    #[wasm_bindgen(js_name = rootName)]
    #[must_use]
    pub fn root_name(&self) -> Option<String> {
        core::root_name(&self.inner)
    }

    /// Evaluate an `XPath` expression and return the matched nodes'
    /// text, as an array of strings.
    ///
    /// Returning text rather than node handles is deliberate: a
    /// `NodeId` is only meaningful against the document that issued
    /// it, and handing an opaque integer across the WASM boundary
    /// invites exactly the misuse the Rust API's lifetime rules
    /// prevent.
    ///
    ///
    /// `namespaces` binds prefixes for the expression, each written
    /// `"PREFIX=URI"` — the same spelling `oxml-cli` takes for `--ns`.
    /// A prefix resolves against these bindings and not against the
    /// document, so one query works across documents that spell the
    /// prefix differently. An unbound prefix is an error.
    ///
    /// # Errors
    ///
    /// Returns a `JsError` if the expression is malformed or uses an
    /// unbound prefix.
    #[wasm_bindgen(js_name = queryText)]
    pub fn query_text(
        &self,
        expression: &str,
        namespaces: Option<Vec<String>>,
    ) -> Result<Vec<String>, JsError> {
        core::query_text(
            &self.inner,
            expression,
            &namespaces.unwrap_or_default(),
        )
        .map_err(|e| JsError::new(&e))
    }

    /// Evaluate an `XPath` expression and return its value as a string.
    ///
    /// Use this for expressions that are not node-sets — `count(..)`,
    /// `string(..)`, a comparison.
    ///
    ///
    /// `namespaces` binds prefixes for the expression, each written
    /// `"PREFIX=URI"` — the same spelling `oxml-cli` takes for `--ns`.
    /// A prefix resolves against these bindings and not against the
    /// document, so one query works across documents that spell the
    /// prefix differently. An unbound prefix is an error.
    ///
    /// # Errors
    ///
    /// Returns a `JsError` if the expression is malformed or uses an
    /// unbound prefix.
    #[wasm_bindgen(js_name = queryValue)]
    pub fn query_value(
        &self,
        expression: &str,
        namespaces: Option<Vec<String>>,
    ) -> Result<String, JsError> {
        core::query_value(
            &self.inner,
            expression,
            &namespaces.unwrap_or_default(),
        )
        .map_err(|e| JsError::new(&e))
    }

    /// How many nodes an expression matches.
    ///
    ///
    /// `namespaces` binds prefixes for the expression, each written
    /// `"PREFIX=URI"` — the same spelling `oxml-cli` takes for `--ns`.
    /// A prefix resolves against these bindings and not against the
    /// document, so one query works across documents that spell the
    /// prefix differently. An unbound prefix is an error.
    ///
    /// # Errors
    ///
    /// Returns a `JsError` if the expression is malformed or uses an
    /// unbound prefix.
    #[wasm_bindgen(js_name = queryCount)]
    pub fn query_count(
        &self,
        expression: &str,
        namespaces: Option<Vec<String>>,
    ) -> Result<usize, JsError> {
        core::query_count(
            &self.inner,
            expression,
            &namespaces.unwrap_or_default(),
        )
        .map_err(|e| JsError::new(&e))
    }

    /// The document as XML.
    ///
    /// The counterpart to `parse`. Reading a document and writing it
    /// back previously meant reaching for `XMLSerializer` and holding
    /// two representations of the same thing.
    ///
    /// Round-trips: the output parses to a document that serialises
    /// identically. It is not guaranteed byte-identical to the input,
    /// because a document has more than one valid spelling -- entity
    /// references and attribute order among them.
    #[wasm_bindgen(js_name = toXml)]
    #[must_use]
    pub fn to_xml(&self) -> String {
        core::to_xml(&self.inner)
    }
}

/// Check whether a document is well-formed, without keeping it.
///
/// Cheaper than `parse` when the answer is all you need, because the
/// tree is dropped immediately.
#[wasm_bindgen(js_name = isWellFormed)]
#[must_use]
pub fn is_well_formed(source: &str) -> bool {
    core::is_well_formed(source)
}
