// What failure looks like, and where it happens.
//
// Run: wasm-pack build --target nodejs && node examples/node/errors.mjs
import assert from "node:assert/strict";
import pkg from "../../pkg/oxml_wasm.js";
const { parse, isWellFormed } = pkg;

// A malformed document throws. The message leads with a line and
// column -- counted in characters, so it is the column an editor shows
// -- then the library's own message and byte offset.
assert.throws(
  () => parse("<a></b>"),
  (e) => {
    assert.match(e.message, /^1:4: /, "leads with line:column");
    assert.match(e.message, /<\/b> closes <a>/);
    return true;
  },
);

// There is no recovery mode and no lenient flag. A parser that guesses
// at a malformed document produces a tree no two implementations agree
// on.
for (const bad of ["<a>", "<a></b>", "", "<a x='1' x='2'/>", "<p:a/>"]) {
  assert.equal(isWellFormed(bad), false, `${bad} should be rejected`);
}

// An expression is compiled when it is used, so a bad one throws from
// the query rather than from parse.
const doc = parse("<a/>");
assert.throws(() => doc.queryText("//["), /.*/);
assert.throws(() => doc.queryValue("//["), /.*/);
assert.throws(() => doc.queryCount("//["), /.*/);

// An external entity is never dereferenced. No file is opened, and no
// option exists to change that.
//
// How the document is *reported* depends on the library version --
// oxml 0.0.3 rejects `&x;` as an unknown entity, 0.0.4 accepts the
// declaration and expands it to nothing -- so this asserts the
// property that matters in both: the file's contents never appear.
const xxe = `<!DOCTYPE d [<!ENTITY x SYSTEM "file:///etc/passwd">]><d>&x;</d>`;
let leaked = "";
try {
  const parsed = parse(xxe);
  leaked = parsed.queryValue("string(/d)");
  parsed.free();
} catch {
  // Rejected outright; nothing was read either way.
}
assert.equal(leaked.trim(), "", "an external entity must never be substituted");
assert.ok(!leaked.includes("root:"), "no file contents");

doc.free();
console.log("errors.mjs: all assertions passed");
