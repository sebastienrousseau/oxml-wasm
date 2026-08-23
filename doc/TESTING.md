<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Testing

Three layers, and the first one carries most of the weight.

## Native tests

```bash
cargo test
```

`src/core.rs` holds the logic and returns `String` errors, so it is
testable without a WebAssembly toolchain. Fourteen tests, including
four that assert the README's own examples produce what the README
says.

Those four exist because two claims were wrong when written: the parse
error message leads with a line and column before the library's byte
offset, and `queryText` on an expression that is not a node-set returns
a one-element array rather than throwing.

## WebAssembly tests

```bash
wasm-pack test --node
```

Confirms the boundary works. Because `lib.rs` is one line per function,
there is little here to get wrong — and what there is, is not
expressible natively.

## The Node examples

```bash
./examples/run-all.sh
```

Builds the package and runs every script in `examples/node/`. They
assert rather than print, so an example that stops working fails CI
instead of quietly making the README wrong.

**Running them found that the entire documented API was wrong.**
`wasm-bindgen` converts snake_case to camelCase, so the real methods
are `queryText`, `rootName` and `isWellFormed`, and `size` is a
property. The README had been written from the Rust source without
building the package.

That is the argument for these examples in one sentence: the mistake
was invisible from the Rust code and obvious the moment anything
executed.

| Script | Covers |
|---|---|
| `basic.mjs` | Every method, predicates, the attribute axis, non-node-set results |
| `errors.mjs` | Malformed documents, invalid expressions, external entities |
| `memory.mjs` | `free()` in a loop, and using a freed document |

`examples/web/index.html` is a browser page with no bundler, run by
hand.

## Version-independent assertions

`errors.mjs` asserts that an external entity's content never appears,
rather than asserting a particular error message. oxml 0.0.3 rejects
the reference as unknown; 0.0.4 accepts the declaration and expands it
to nothing. Neither reads the file, and the security property is what
matters.
