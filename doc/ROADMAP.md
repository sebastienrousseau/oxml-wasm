<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Roadmap

## Where this is

WebAssembly bindings for `oxml`: parse a document, ask XPath 1.0
questions of it, get strings back.

Being pure Rust with no C dependency is what makes this possible at
all. `libxml`-based crates cannot target WebAssembly without a libxml2
toolchain, which is most of the reason a browser-side XPath has not
been practical in Rust.

The surface is deliberately small. JavaScript already has `DOMParser`;
what it does not have is a fast, dependency-free XPath that behaves
identically to the server-side one. That is what these bindings are
for.

The crate is two layers: `core`, which is plain Rust and where every
decision is made, and a thin `#[wasm_bindgen]` translation above it.
`core` is tested natively; the bindings are tested through
`wasm-pack test --node`, because they need a JavaScript runtime to
hand values to.

## The order

**1. Serialisation.** `oxml` gained `to_xml()` at 0.0.8 and these
bindings do not expose it, so "you need to produce XML" is still a
reason not to use this. It is the smallest gap with the clearest
demand: a caller that can read but not write has to reach for
`XMLSerializer` and hold two representations.

**2. Node handles rather than strings.** Every query returns strings.
A caller that wants to ask a follow-up question about a matched node
has to re-query from the root with a longer expression. Returning an
opaque handle that later calls accept would fix that, and is bounded
work now that `Document` already crosses the boundary.

**3. Bounded parsing for the heap ceiling.** WebAssembly linear memory
has a ceiling, and the whole tree is built in it. `oxml`'s limits API
can refuse a document before it exhausts that, which turns a crashed
module into an error a caller can handle.

## What is deliberately absent

**A DOM.** This returns strings and, later, handles. `DOMParser`
returns nodes you can walk and mutate, it is already in the browser,
and it costs no download. Reimplementing it in WebAssembly would be
slower and larger for the cases it already covers.

**Streaming.** `oxml` has a streaming reader, but the value here is
XPath, and XPath needs the tree.

## Non-goals

Replacing `DOMParser`. If `DOMParser` is enough, it is enough -- the
README says so in its own "when not to use" section, and that should
stay true.
