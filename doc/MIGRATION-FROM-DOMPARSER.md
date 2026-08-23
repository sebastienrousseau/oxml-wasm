<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Migrating from `DOMParser`

## Should you?

`DOMParser` is already in the browser and costs no download. Reach for
this when:

- You need behaviour that does not vary between browsers.
- The same queries must run in Node and in a browser and give
  identical answers.
- You want a parse failure to *throw*, with a position, rather than
  returning a document containing a `<parsererror>` element whose shape
  differs by browser.

Stay with `DOMParser` when you need a DOM you can walk and modify, or
when you are doing one simple extraction.

## Command for command

| Browser API | `oxml-wasm` |
|---|---|
| `new DOMParser().parseFromString(s, "text/xml")` | `parse(s)` |
| looking for `<parsererror>` in the result | `parse` throws; or `isWellFormed(s)` |
| `doc.documentElement.tagName` | `doc.rootName()` |
| `doc.evaluate(e, doc, null, XPathResult.ORDERED_NODE_ITERATOR_TYPE, null)` then iterating | `doc.queryText(e)` |
| `doc.evaluate(e, doc, null, XPathResult.NUMBER_TYPE, null).numberValue` | `doc.queryValue(e)` |
| `doc.evaluate(e, doc, null, XPathResult.BOOLEAN_TYPE, null).booleanValue` | `doc.queryValue(e)` |
| a namespace resolver argument | not supported |
| `node.textContent` | included in `queryText`'s result |
| mutating the document | not supported |

## Detecting a parse failure

```javascript
// DOMParser
const doc = new DOMParser().parseFromString(xml, "text/xml");
if (doc.querySelector("parsererror")) { /* … */ }

// oxml-wasm
try {
  const doc = parse(xml);
} catch (e) {
  console.error(e.message);  // 1:4: at byte 3: </b> closes <a>
}
```

## Namespaces

`document.evaluate` takes a namespace resolver. `oxml-wasm` has no
equivalent, and prefixes in an expression are **not** resolved against
the document's bindings — `//x:item` selects every `item`. Filter
explicitly:

```javascript
doc.queryText("//*[namespace-uri()='urn:example' and local-name()='item']");
```

## Remember to free

`DOMParser` documents are garbage-collected. These are not, reliably —
see [MEMORY.md](MEMORY.md).
