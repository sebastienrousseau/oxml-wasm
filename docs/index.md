---
title: oxml-wasm — WebAssembly XML Toolkit
description: WebAssembly bindings for oxml: XML parsing and XPath in the browser and Node.js.
hide:
  - navigation
  - toc
---

<section class="dot-hero" markdown>

# oxml-wasm

<p class="tagline">WebAssembly bindings for oxml — fast XML parsing, DOM inspection, and XPath 1.0 querying in the browser and Node.js.</p>

<div class="buttons">
  <a class="primary" href="ARCHITECTURE/">Architecture →</a>
  <a href="https://github.com/sebastienrousseau/oxml-wasm">GitHub</a>
  <a href="MEMORY/">Memory Model</a>
  <a href="MIGRATION-FROM-DOMPARSER/">DOMParser Migration</a>
</div>

</section>

## What's inside

<div class="grid cards" markdown>

- :material-language-javascript:{ .lg .middle } **TypeScript & JavaScript**

    ---

    Zero-overhead wasm-bindgen generated bindings with full TypeScript definitions (`.d.ts`) included out of the box.

    [→ Architecture](ARCHITECTURE.md)

- :material-memory:{ .lg .middle } **Linear memory safety**

    ---

    Arena-backed memory model designed to operate deterministically inside WebAssembly linear memory without leaks.

    [→ Memory Model](MEMORY.md)

- :material-swap-horizontal:{ .lg .middle } **DOMParser replacement**

    ---

    Faster, standard-compliant alternative to browser `DOMParser` with full XPath 1.0 support across all environments.

    [→ Migration Guide](MIGRATION-FROM-DOMPARSER.md)

- :material-shield-check:{ .lg .middle } **Zero `unsafe` code**

    ---

    Built purely from oxml's `#![forbid(unsafe_code)]` core engine, verified under strict security bounds.

    [→ Assurance Case](ASSURANCE-CASE.md)

</div>

## Quick start

Install via npm:

```bash
npm install oxml-wasm
```

Use in JavaScript or TypeScript:

```javascript
import { Document } from "oxml-wasm";

const xml = `<catalog><book id="1"><title>Rust & WebAssembly</title></book></catalog>`;

const doc = Document.parse(xml);
const titles = doc.xpath("//book/title");
console.log(titles[0].textContent());
```

## Where to next

- [**Architecture**](ARCHITECTURE.md) — WebAssembly architecture, bindings layout, and bundle sizes.
- [**Memory Model**](MEMORY.md) — Heap allocation, handle life cycles, and garbage collection integration.
- [**Migration from DOMParser**](MIGRATION-FROM-DOMPARSER.md) — Upgrading browser XML parsing workflows.
- [**Assurance Case**](ASSURANCE-CASE.md) — Security boundaries and untrusted payload isolation.
- [**Testing**](TESTING.md) — Node.js and headless browser test runners.

## Current release

- Release notes: [GitHub Releases](https://github.com/sebastienrousseau/oxml-wasm/releases)
- Crates.io: [crates.io/crates/oxml-wasm](https://crates.io/crates/oxml-wasm)
- Repository: [sebastienrousseau/oxml-wasm](https://github.com/sebastienrousseau/oxml-wasm)
