// Every method, with assertions.
//
// Run: wasm-pack build --target nodejs && node examples/node/basic.mjs
//
// These assert rather than print. An example that prints goes stale
// the moment behaviour changes and still looks correct; one that
// asserts fails CI instead.
import assert from "node:assert/strict";
// `wasm-pack build --target nodejs` emits CommonJS, so the named
// exports are reached through the default import.
import pkg from "../../pkg/oxml_wasm.js";
const { parse, isWellFormed } = pkg;

const CATALOGUE = `
  <catalogue>
    <book lang="en"><title>Dune</title><price>9.99</price></book>
    <book lang="fr"><title>Germinal</title><price>7.50</price></book>
  </catalogue>`;

const doc = parse(CATALOGUE);

assert.equal(doc.rootName(), "catalogue");
assert.ok(doc.size > 0, "a parsed document has nodes");

// queryText: the string-value of each matched node.
assert.deepEqual(doc.queryText("//title"), ["Dune", "Germinal"]);

// queryCount: how many nodes matched.
assert.equal(doc.queryCount("//book"), 2);

// queryValue: any expression, converted to a string by XPath's rules.
assert.equal(doc.queryValue("sum(//price)"), "17.49");
assert.equal(doc.queryValue("count(//book)"), "2");
assert.equal(doc.queryValue("string(//title)"), "Dune");

// Predicates work as specified.
assert.deepEqual(doc.queryText('//book[@lang="en"]/title'), ["Dune"]);

// Attributes are nodes, so they have a string-value.
assert.deepEqual(doc.queryText("//book/@lang"), ["en", "fr"]);

// queryText on a non-node-set returns one element rather than
// throwing; queryCount returns 0, because nothing was matched.
assert.deepEqual(doc.queryText("count(//book)"), ["2"]);
assert.equal(doc.queryCount("count(//book)"), 0);

// isWellFormed parses and discards, for a yes-or-no answer.
assert.equal(isWellFormed("<a/>"), true);
assert.equal(isWellFormed("<a>"), false);

doc.free();
console.log("basic.mjs: all assertions passed");
