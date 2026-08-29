# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.8] - 2026-08-29

### Added

- A fuzz target over the exported operations, run for 300 seconds on
  every pull request.

- **The examples are measured against the public API** of `core`,
  where the deciding logic lives. The `#[wasm_bindgen]` layer is
  excluded per-function -- it needs a JavaScript runtime and is
  covered by the Node examples -- so a plain `pub fn` added beside it
  is still checked.

### Security

- **`wasm-pack` is installed through a pinned action** rather than
  `curl -sSf https://... | sh`. The script was fetched over the
  network and executed unverified on every run, so whatever it
  contained that day became part of the build.

- **`cargo audit` and `cargo deny` now actually run.** The badge said
  they did; only `oxml` had the workflow.

- Every action pinned by commit SHA, branch coverage gated, CodeQL
  added, and the DCO enforced.

## [0.0.7] - 2026-08-28

### Added

- **Benchmarks** for the exported operations, measuring `src/core.rs`
  natively rather than through WebAssembly. That isolates the parsing
  and query work from the JavaScript boundary, so a change in one is
  not read as a change in the other -- and it is why the figures say
  what they cannot tell you.

- A **gate script** (`./scripts/gate.sh`) running everything CI runs,
  including the `wasm-pack` steps. It skips those *loudly* when the
  tool is absent, counting that as a failure: a check that vanishes
  with its tool reports success over what it did not run.

- The Node examples now **run in CI**. `basic.mjs` opens by saying "an
  example that asserts fails CI instead" and the README listed them as
  "Runnable, and run in CI". Nothing ran them.

### Changed

- Built on oxml 0.0.7, which reads a document from any `BufRead`. The
  suite ships one version number across all six crates.

- `core` is public, so the benchmark can reach it.

- The README now follows the same shape as the rest of the suite, and
  gained the Benchmarks, Documentation and Acknowledgements sections
  it lacked.

## [0.0.6] - 2026-08-26

### Changed

- Built on oxml 0.0.6 and xmlschema 0.0.6. The suite ships one version
  number across all six crates.

  xmlschema 0.0.6 is the substantial half of this release: its W3C
  conformance pass rate moved from 71.7% to 95.6%, and its coverage of
  the suite -- the share of tests that produce an answer meaning
  anything -- from 27.0% to 87.6%. Schemas this crate previously read
  as valid, and did not enforce, are now either enforced or reported
  as unenforceable.

## [0.0.5] - 2026-08-24

### Changed

- Built on oxml 0.0.5, which completes `XPath` 1.0: all thirteen axes
  and all 27 functions.

  **One behaviour change reaches expressions passed through this
  crate.** A function name outside the specification's library, or a
  call with the wrong number of arguments, used to compile and evaluate
  to an empty node-set. Both are now compile errors, reported with an
  offset. `starts-with("abc")` answered `true` before, because the
  absent argument read as the empty string.

  Six functions that previously answered `""` now work:
  `substring-before`, `substring-after`, `translate`, `name`, `id` and
  `lang`. So do the `following`, `preceding` and `namespace` axes.

## [0.0.4] - 2026-08-24

### Added

- An optional array of `"PREFIX=URI"` bindings on `queryText`,
  `queryValue` and `queryCount`. oxml 0.0.4 resolves prefixes in
  `XPath` name tests instead of matching on the local part alone.

## [0.0.3] - 2026-08-22

### Added

- Initial release. WebAssembly bindings for oxml: XML parsing and XPath in the browser
- Tracks the version line of the [`oxml`](https://github.com/sebastienrousseau/oxml)
  core, so a given version of any suite member is built and tested against
  the matching core.

[0.0.3]: https://github.com/sebastienrousseau/oxml-wasm/releases/tag/v0.0.3
