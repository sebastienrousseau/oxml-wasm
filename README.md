<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">oxml-wasm</h1>

<p align="center">
  WebAssembly bindings for <a href="https://github.com/sebastienrousseau/oxml">oxml</a> — XML parsing and XPath in the browser.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/oxml-wasm/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/oxml-wasm/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/oxml-wasm"><img src="https://img.shields.io/crates/v/oxml-wasm.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/oxml-wasm"><img src="https://img.shields.io/badge/docs.rs-oxml-wasm-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/oxml-wasm"><img src="https://img.shields.io/badge/lib.rs-oxml-wasm-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/oxml-wasm"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/oxml-wasm?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Why this is possible

`libxml`-based crates cannot target WebAssembly without a libxml2
toolchain, which is most of the reason a browser-side XPath has not
been practical in Rust. Being pure Rust with no C dependency is what
makes these bindings work at all.

## Build

```bash
wasm-pack build --target web
```

## Usage

```javascript
import init, { parse, isWellFormed } from "./pkg/oxml_wasm.js";

await init();

const doc = parse(`
  <library>
    <book lang="en"><title>Dune</title></book>
    <book lang="fr"><title>Germinal</title></book>
  </library>
`);

doc.queryText("//book[@lang='en']/title");  // ["Dune"]
doc.queryValue("count(//book)");             // "2"
doc.queryCount("//book");                    // 2
doc.rootName;                                // "library"

isWellFormed("<a></b>");                     // false
```

## API

| Export | What it does |
|---|---|
| `parse(source)` | Parse a document; throws with line and column if malformed |
| `isWellFormed(source)` | Check without keeping the tree |
| `doc.queryText(expr)` | Matched nodes' text, as an array |
| `doc.queryValue(expr)` | The expression's value as a string |
| `doc.queryCount(expr)` | How many nodes matched |
| `doc.rootName` | The root element's name |
| `doc.size` | Node count |

## Design

The surface is deliberately small. JavaScript already has `DOMParser`;
what it does not have is a fast, dependency-free XPath that behaves
identically to the server-side one.

Queries return text rather than node handles. A node handle is only
meaningful against the document that issued it, and passing an opaque
integer across the WASM boundary invites exactly the misuse the Rust
API's type system prevents.

## The oxml suite

Every member ships the **same version number**, so there is never a
compatibility table to consult. Versions advance in `0.0.1` steps along
the `0.0.x` line; `0.1.0` follows `0.0.999`.

| Crate | What it is |
|---|---|
| [`oxml`](https://github.com/sebastienrousseau/oxml) | Core — parser, tree, XPath 1.0 |
| [`oxml-cli`](https://github.com/sebastienrousseau/oxml-cli) | Command-line querying and validation |
| [`oxml-lsp`](https://github.com/sebastienrousseau/oxml-lsp) | Diagnostics for editors |
| [`oxml-mcp`](https://github.com/sebastienrousseau/oxml-mcp) | Model Context Protocol server |
| [`oxml-wasm`](https://github.com/sebastienrousseau/oxml-wasm) | WebAssembly bindings |
| [`xmlschema`](https://github.com/sebastienrousseau/xmlschema) | XSD validation |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). By participating you agree to
the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
