<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Memory

## The problem

A `Document` holds memory inside the WebAssembly instance's linear
memory, not on the JavaScript heap.

The JavaScript garbage collector will eventually collect the wrapper
object, but it has no idea the wrapper stands for several megabytes on
the other side of the boundary. It sees a small object and is in no
hurry. Meanwhile the WebAssembly heap only grows.

For one small document this does not matter. For a page that parses a
document per request, it is a leak.

## The fix

```javascript
const doc = parse(largeXml);
try {
  const titles = doc.queryText("//title");
  // …
} finally {
  doc.free();
}
```

`finally`, not just at the end: an exception in the middle otherwise
leaks the document.

On a runtime with explicit resource management, the generated class
implements `Symbol.dispose`:

```javascript
using doc = parse(largeXml);
const titles = doc.queryText("//title");
```

## After `free()`

Calling a method on a freed document throws. It does not read released
memory — `wasm-bindgen` null-checks the handle. This is asserted in
[`examples/node/memory.mjs`](../examples/node/memory.mjs).

## How much memory a document takes

The library performs about 4.1 allocations per node and holds the whole
tree. There is no streaming mode, and WebAssembly linear memory has a
ceiling — 4 GiB for `wasm32`, and in practice much less depending on
the runtime.

If a document might be very large, check its size before parsing.

## Reducing the module size

Building without the XPath feature removes the entire expression
engine. If you only need `parse` and `isWellFormed`, that is a
worthwhile saving.

Measure your own build; a figure quoted without the target, the
`wasm-opt` settings and the bundler's tree-shaking is not useful:

```bash
wasm-pack build --target web --release
ls -l pkg/*.wasm
```
