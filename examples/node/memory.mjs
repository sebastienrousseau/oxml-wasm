// Freeing documents.
//
// Run: wasm-pack build --target nodejs && node examples/node/memory.mjs
//
// A Document holds memory inside the WebAssembly instance, not on the
// JavaScript heap. The garbage collector does not know how large it is
// and will not run on its account, so a page that parses repeatedly
// leaks unless it frees.
import assert from "node:assert/strict";
import pkg from "../../pkg/oxml_wasm.js";
const { parse } = pkg;

function build(items) {
  const rows = Array.from(
    { length: items },
    (_, i) => `<item id="i${i}"><name>Item ${i}</name></item>`,
  ).join("");
  return `<catalogue>${rows}</catalogue>`;
}

const source = build(5_000);

// The pattern: free in a `finally`, so an exception in the middle does
// not leak the document.
for (let round = 0; round < 20; round++) {
  const doc = parse(source);
  try {
    assert.equal(doc.queryCount("//item"), 5_000);
  } finally {
    doc.free();
  }
}

// Using a freed document throws rather than reading released memory.
const doc = parse("<a/>");
doc.free();
assert.throws(() => doc.queryCount("//a"), /.*/);

console.log("memory.mjs: all assertions passed");
