# Security Policy

## Supported versions

oxml-wasm is pre-1.0. Only the most recent `0.0.x` release receives security
fixes.

## Reporting a vulnerability

Please report privately, not as a public issue:

- GitHub's [private vulnerability reporting](https://github.com/sebastienrousseau/oxml-wasm/security/advisories/new)
- or email <sebastian.rousseau@gmail.com>

You should get an acknowledgement within 72 hours.

## Threat model

The bindings parse caller-supplied XML with `Limits::strict()` and use
fresh default XPath evaluation budgets for each query. Source and decoded
document sizes are limited to 8 MiB. A budget bounds parser/evaluator work,
not all JavaScript allocations or all WebAssembly runtime memory.

Internal entities can expand within parser budgets. No external document
or entity is fetched automatically. The embedding runtime must enforce its
own linear-memory and execution limits; sandboxing does not prevent denial
of service by itself.

**Memory safety.** This crate forbids unsafe Rust in its own source.
Dependencies and runtimes have separate safety obligations. This
does not rule out resource exhaustion, logic defects or protocol errors.

## What is not a vulnerability

- Rejecting a document other parsers accept, when the document is not
  well-formed. Please still report it — it is a bug, just not a
  security one.
