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

## Fuzzing

```bash
cargo +nightly fuzz run core
```

`core` and `query` covers the exported operations, which sit directly behind `#[wasm_bindgen]` where a panic is a WebAssembly trap.

1,467,446 and 983,477 executions have run without a crash. CI runs the target for
300 seconds on every pull request, seeded from the tracked files in
`fuzz/seeds/` — the grown corpus is build output and is not tracked,
so a run starts from the same place every time rather than from
whatever a previous run happened to discover. A crash input is
uploaded as a build artefact, because knowing only that something
broke is not much use.

## Coverage

Line coverage is gated in CI at a 95% floor. **Branch coverage is
100%**, gated at 80.

Branch coverage needs a nightly toolchain: `cargo llvm-cov --branch`
does not build on the version this project pins. It was recorded as
unmeasurable for a while on the strength of that one failure, which
was a conclusion drawn from a single attempt.

