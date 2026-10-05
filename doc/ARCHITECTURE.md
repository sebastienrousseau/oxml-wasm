<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Architecture

Two files, and the split between them is the whole design.

```
src/core.rs   everything the bindings do, in plain Rust
src/lib.rs    a #[wasm_bindgen] layer, one line per function
```

## Why the logic is not in the binding layer

`JsError` can only be constructed meaningfully against a JavaScript
runtime. A test that had to build one could not run under `cargo test`
— it would need `wasm-pack test`, a browser or Node, and a
WebAssembly toolchain.

So the logic lives in `core.rs` and returns `String` errors. It is
tested natively, in milliseconds, by anyone with a Rust toolchain.
`lib.rs` maps each function to its `JsError` equivalent, one line each,
with nothing left in it to get wrong.

The result: 14 native tests cover the behaviour, and the WebAssembly
tests only have to confirm the boundary works.

## Names change at the boundary

`wasm-bindgen` converts snake_case to camelCase, so `core::query_text`
is `queryText` in JavaScript, and `size` becomes a property rather than
a method.

This is worth stating plainly because it is not visible from the Rust
source, and documenting the Rust names as though they were the
JavaScript API is an easy mistake — this README made exactly that
mistake before the examples were run against a real build.

Read `pkg/oxml_wasm.d.ts` after building; it is the authority.

## Strings, not nodes

`queryText` returns `string[]`. There is no node handle.

Every value crossing the boundary is copied, and a node is only
meaningful with the document that issued it. Returning handles would
mean a call into WebAssembly for every property access — more
expensive than returning the strings the caller was going to ask for,
and it invites exactly the cross-document misuse that the Rust API's
types prevent.

## Three query methods, not one

`queryText`, `queryValue` and `queryCount` are separate rather than one
method returning a tagged union, because a union costs a serialisation
on every call regardless of what the caller wanted. A caller who needs
a count should pay for a number.
