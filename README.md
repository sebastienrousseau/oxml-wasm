<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">oxml-wasm</h1>

<p align="center">
  XML parsing and XPath 1.0 in the browser — WebAssembly bindings for
  <a href="https://github.com/sebastienrousseau/oxml">oxml</a>, with no C
  toolchain and zero <code>unsafe</code> code.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/oxml-wasm/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/oxml-wasm/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/oxml-wasm"><img src="https://img.shields.io/crates/v/oxml-wasm.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/oxml-wasm"><img src="https://img.shields.io/badge/docs.rs-oxml--wasm-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
</p>

---

## Contents

- [Why this is possible at all](#why-this-is-possible-at-all)
- [Install](#install)
- [Quick Start](#quick-start)
- [The oxml ecosystem](#the-oxml-ecosystem)
- [API](#api)
- [Memory and lifetimes](#memory-and-lifetimes)
- [Errors](#errors)
- [Migration](#migration) — from `DOMParser`
- [Ecosystem comparison](#ecosystem-comparison)
- [Capabilities in 0.0.4](#capabilities-in-004)
- [Bundle size](#bundle-size)
- [Examples](#examples)
- [When not to use oxml-wasm](#when-not-to-use-oxml-wasm)
- [FAQ](#faq)
- [Development](#development)
- [Security](#security)
- [License](#license)

---

## Why this is possible at all

`libxml`-based crates cannot target WebAssembly without a libxml2
toolchain, which is most of the reason a browser-side XPath has not
been practical in Rust. `oxml` is pure Rust with no C dependency and no
`unsafe`, so it compiles to `wasm32-unknown-unknown` the same way any
other crate does.

That is the whole argument for this package. Everything else follows.

## Install

```bash
npm install oxml-wasm
```

Or build it yourself:

```bash
wasm-pack build --target web      # for a browser
wasm-pack build --target nodejs   # for Node
wasm-pack build --target bundler  # for webpack, Vite, Rollup
```

## Quick Start

```javascript
import init, { parse, is_well_formed } from "oxml-wasm";

await init();

const doc = parse(`
  <catalogue>
    <book lang="en"><title>Dune</title><price>9.99</price></book>
    <book lang="fr"><title>Germinal</title><price>7.50</price></book>
  </catalogue>
`);

doc.rootName();                       // "catalogue"
doc.size;                              // node count (a property)
doc.queryText("//title");             // ["Dune", "Germinal"]
doc.queryCount("//book");             // 2
doc.queryValue("sum(//price)");       // "17.49"

isWellFormed("<a>");                 // false
```

## The oxml ecosystem

| Crate | What it is |
|---|---|
| [`oxml`](https://github.com/sebastienrousseau/oxml) | The library: parser, tree, XPath 1.0 |
| [`xmlschema`](https://github.com/sebastienrousseau/xmlschema) | XSD validation |
| [`oxml-cli`](https://github.com/sebastienrousseau/oxml-cli) | The command line |
| **`oxml-wasm`** | **This crate — WebAssembly** |
| [`oxml-mcp`](https://github.com/sebastienrousseau/oxml-mcp) | Model Context Protocol server |
| [`oxml-lsp`](https://github.com/sebastienrousseau/oxml-lsp) | Language Server Protocol server |

All six ship one version number, in steps of 0.0.1.

## API

### `parse(source: string): Document`

Parses a document. Throws on malformed input; the message carries the
byte offset and what went wrong.

### `isWellFormed(source: string): boolean`

Parses and discards, returning whether it succeeded. Use this when you
want a yes-or-no answer and do not need the tree — it avoids handing a
`Document` back across the boundary.

### `Document`

| Method | Returns |
|---|---|
| `size` | Number of nodes (a property, not a method) |
| `rootName()` | The root element's local name, or `undefined` |
| `queryText(expr)` | `string[]` — the string-value of each matched node |
| `queryValue(expr)` | `string` — the expression's value, converted to a string |
| `queryCount(expr)` | `number` — how many nodes matched |

`queryText` is for node-sets. `queryValue` is for anything —
`count(//x)`, `sum(//price)`, `string(//title)`, a boolean — and
applies XPath's own conversion rules.

`queryText` on an expression that is *not* a node-set does not throw:
it returns a one-element array holding the converted value, so a caller
iterating the result still gets something. `queryCount` on the same
expression returns 0, because nothing was matched.

The three are separate rather than one method returning a tagged union,
because crossing the WebAssembly boundary costs a serialisation and a
union costs one on every call regardless of what the caller wanted.

> **Method names are camelCase in JavaScript.** `wasm-bindgen`
> converts `query_text` in the Rust source to `queryText` in the
> generated module. If you are reading the Rust code, add the
> conversion mentally.

## Memory and lifetimes

A `Document` holds memory inside the WebAssembly instance, not on the
JavaScript heap. `wasm-bindgen` cannot collect it for you: the
JavaScript garbage collector does not know how large it is and will not
run on its account.

**Call `free()` when you are done with a large document:**

```javascript
const doc = parse(largeXml);
try {
  const titles = doc.queryText("//title");
  // …
} finally {
  doc.free();
}
```

The generated class also implements `Symbol.dispose`, so on a runtime
with explicit resource management the `try`/`finally` collapses to:

```javascript
using doc = parse(largeXml);
const titles = doc.queryText("//title");
```

For a handful of small documents this does not matter. For a page that
parses a document per request, it is the difference between steady
memory and a slow leak.

## Errors

`parse` throws a `JsError`. The message is the library's, which
includes the byte offset:

```javascript
try {
  parse("<a></b>");
} catch (e) {
  console.error(e.message);  // 1:4: at byte 3: </b> closes <a>
}
```

The message leads with a line and column, counted in characters, then
the library's own message with its byte offset.

An invalid XPath expression throws from `query_*` rather than at parse
time, because the expression is compiled when it is used.

## Migration

### From `DOMParser` and `document.evaluate`

| Browser API | `oxml-wasm` |
|---|---|
| `new DOMParser().parseFromString(s, "text/xml")` | `parse(s)` |
| checking for `<parsererror>` in the result | `parse` throws; or `isWellFormed(s)` |
| `doc.documentElement.tagName` | `doc.rootName()` |
| `doc.evaluate(e, doc, null, XPathResult.ORDERED_NODE_ITERATOR_TYPE, null)` | `doc.queryText(e)` |
| `doc.evaluate(e, doc, null, XPathResult.NUMBER_TYPE, null).numberValue` | `doc.queryValue(e)` |
| namespace resolver argument | not supported — see the FAQ |

The most useful difference is error reporting. `DOMParser` signals a
failure by returning a document containing a `<parsererror>` element,
whose shape differs between browsers; `parse` throws with a byte
offset.

The most awkward difference is that you get strings back, not nodes.
There is no DOM here to hand you.

## Ecosystem comparison

| | `oxml-wasm` | `DOMParser` | `fast-xml-parser` | `libxmljs` |
|---|---|---|---|---|
| Runs in a browser | Yes | Yes | Yes | No |
| Runs in Node | Yes | With a shim | Yes | Yes |
| XPath 1.0 | Yes | Yes | No | Yes |
| Returns a DOM | No | Yes | No (objects) | Yes |
| Consistent across browsers | Yes | No | Yes | n/a |
| Native code | Wasm | Browser | JavaScript | C++ addon |
| Fetches external entities | **Never** | No | No | Configurable |

`DOMParser` is already there and costs nothing to use. Reach for this
when you need behaviour that does not vary by browser, or when the same
queries have to run in Node and in a browser and give identical
answers.

## Capabilities in 0.0.4

- Full XML 1.0 and 1.1 parsing, namespaces resolved by URI
- XPath 1.0: ten axes, 25 functions, all four value types
- Well-formedness checking without building a tree
- Node counts and root element name
- No C toolchain, no `unsafe`, no network access

**Not yet:** returning nodes rather than strings, mutation,
serialisation, XSD validation, streaming, namespace prefix bindings.

## Bundle size

Not published here. A figure depends on the `wasm-pack` target,
`wasm-opt` settings, whether the XPath feature is enabled and what your
bundler strips, and quoting one number without all four is misleading.

Measure yours:

```bash
wasm-pack build --target web --release
ls -l pkg/*.wasm
```

Building without XPath, if you only need parsing and well-formedness
checks, removes the whole expression engine.

## Examples

Runnable, and run in CI:

| Example | What it shows |
|---|---|
| [`examples/node/basic.mjs`](examples/node/basic.mjs) | Every method, with assertions |
| [`examples/node/errors.mjs`](examples/node/errors.mjs) | Malformed documents and invalid expressions |
| [`examples/node/memory.mjs`](examples/node/memory.mjs) | `free()`, and what happens without it |
| [`examples/web/index.html`](examples/web/index.html) | A browser page with no bundler |

## When not to use oxml-wasm

- **You need a DOM.** This returns strings. `DOMParser` returns nodes
  you can walk and modify.
- **`DOMParser` is enough.** It is already in the browser and costs no
  download.
- **You are only reading a small, known document.** A WebAssembly
  module is a lot of machinery for one `querySelector`-shaped job.
- **You need to produce XML.** No serialiser.
- **Documents are larger than the Wasm heap.** The whole tree is built
  in memory, inside a linear memory that has a ceiling.

## FAQ

### Why strings instead of nodes?

Because every value crossing the WebAssembly boundary is copied, and a
node is only useful with the document it came from. Handing back node
handles would mean a call into Wasm for every property access — far
more expensive than returning the strings you were going to ask for
anyway.

If you need the structure, query for the parts you need.

### How do I query a document with namespaces?

**This changes at oxml 0.0.4, and these bindings do not expose the fix
yet.**

In the version this package currently links, a prefix in an expression
is not resolved: `//x:item` selects every `item` regardless of
namespace. Filter on the URI:

```javascript
doc.queryText("//*[namespace-uri()='urn:example' and local-name()='item']");
```

From oxml 0.0.4 a prefixed name test resolves against bindings supplied
with the query, an unbound prefix is a compile error, and an
unprefixed name test matches only nodes in no namespace — which is
XPath 1.0, and the same rule `document.evaluate` follows.

Exposing that needs a second argument on the query methods, which is
not implemented. The `namespace-uri()` form above works in both
versions and is not going away.

### Do I have to call `free()`?

Not for small documents. For large ones, or for a page that parses
repeatedly, yes — see [Memory and lifetimes](#memory-and-lifetimes).

### Does it work in a Web Worker?

Yes, and it is a good place for it: parsing a large document blocks
whatever thread it runs on.

### Can it fetch a document over HTTP?

No. There is no network code in the module. Fetch it in JavaScript and
pass the string.

### Is it safe against XXE?

Yes, structurally. The parser contains no code that opens a file or a
socket, so an external entity is never substituted with anything.

How it is *reported* depends on the library version: oxml 0.0.3
rejects the reference as an unknown entity, and 0.0.4 accepts the
declaration and expands it to nothing. Neither reads the file. The
example in `examples/node/errors.mjs` asserts the property rather than
the message, so it holds across both.

### Why is `parse` throwing on a document my browser accepts?

Browsers are lenient in places the specification is not. `oxml` rejects
malformed documents rather than guessing — the message says which rule
and where. If you believe the document is well-formed and it is
rejected, that is a bug worth reporting.

### Does it support XPath 2.0?

No. XPath 1.0, which is also what `document.evaluate` implements.

### Which browsers?

Anything with WebAssembly: every current browser. There is no
polyfilled fallback.

## Development

```bash
cargo test                    # the core logic, natively
wasm-pack test --node         # the bindings
wasm-pack build --target web
node examples/node/basic.mjs
```

The logic lives in `src/core.rs` and is tested natively, so most of the
behaviour is covered without a WebAssembly toolchain. `src/lib.rs` is
the thin `#[wasm_bindgen]` layer over it.

## Security

No network access, no filesystem access, external entities never
dereferenced, entity expansion bounded, `#![forbid(unsafe_code)]`.

A WebAssembly module is sandboxed by the runtime as well, so the worst
a hostile document can do is exhaust the module's memory.

See
<https://github.com/sebastienrousseau/oxml/blob/main/doc/SECURITY-MODEL.md>.

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](LICENSE-APACHE))
or MIT ([LICENSE-MIT](LICENSE-MIT)), at your option.
