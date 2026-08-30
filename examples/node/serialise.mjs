// Writing a document back out, from JavaScript.
//
// Run: wasm-pack build --target nodejs && node examples/node/serialise.mjs
//
// These assert rather than print. An example that prints goes stale
// the moment behaviour changes and still looks correct; one that
// asserts fails CI instead.
import assert from "node:assert/strict";
// `wasm-pack build --target nodejs` emits CommonJS, so the named
// exports are reached through the default import.
import pkg from "../../pkg/oxml_wasm.js";
const { parse } = pkg;

const CATALOGUE = `<catalogue><book lang="en"><title>Dune</title></book></catalogue>`;

const doc = parse(CATALOGUE);
const xml = doc.toXml();

// A fixed point, not a byte-identical copy. A document has more than
// one valid spelling — entity references and attribute order among
// them — so the guarantee is that parsing the output and serialising
// it again produces the same text.
const again = parse(xml);
assert.equal(again.toXml(), xml, "serialisation must be a fixed point");

// The content survives, which is the part a caller depends on.
assert.equal(again.queryValue("string(//title)"), "Dune");
assert.equal(again.rootName(), "catalogue");

// Escaping has to hold, or the second parse would read markup where
// the first read characters.
const tricky = parse(`<a note="x &lt; y">p &amp; q</a>`);
const out = tricky.toXml();
assert.ok(out.includes("&amp;"), `text & must stay escaped: ${out}`);
assert.ok(out.includes("&lt;"), `attribute < must stay escaped: ${out}`);
assert.equal(
  parse(out).queryValue("string(//a)"),
  "p & q",
  "the text must survive the round trip unchanged",
);

console.log("serialise.mjs: all assertions passed");
