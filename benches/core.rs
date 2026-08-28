// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-wasm. All rights reserved.

//! What the exported operations cost.
//!
//! These are called from JavaScript, usually in a loop over a set of
//! documents or a set of expressions, so the figure that matters is
//! per call rather than per megabyte.
//!
//! Absolute numbers describe the machine as much as the code — see
//! `oxml`'s `doc/BENCHMARKS.md`. Compare runs, not numbers.

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

fn document(n: usize) -> String {
    let mut s = String::from("<catalogue>");
    for i in 0..n {
        let _ = write!(
            s,
            "<item id=\"i{i}\"><name>Item {i}</name><price>{i}.99</price></item>"
        );
    }
    s.push_str("</catalogue>");
    s
}

/// The fastest of `rounds` runs: contention can only make a run
/// slower, so the fastest is the least perturbed sample.
fn fastest(rounds: usize, mut f: impl FnMut()) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..rounds {
        let start = Instant::now();
        f();
        best = best.min(start.elapsed().as_secs_f64());
    }
    best
}

fn main() {
    let doc = document(2_000);
    let parsed = oxml_wasm::core::parse(&doc).expect("well-formed");
    let no_ns: Vec<String> = Vec::new();

    println!(
        "oxml-wasm core, fastest of 20 runs ({} KB)\n",
        doc.len() / 1024
    );

    let t = fastest(20, || {
        let _ = black_box(oxml_wasm::core::parse(black_box(&doc)));
    });
    println!("  parse                     {:>8.3} ms", t * 1e3);

    let t = fastest(20, || {
        let _ = black_box(oxml_wasm::core::is_well_formed(black_box(&doc)));
    });
    println!("  is_well_formed            {:>8.3} ms", t * 1e3);

    let t = fastest(20, || {
        let _ = black_box(oxml_wasm::core::query_count(
            black_box(&parsed),
            "//item",
            &no_ns,
        ));
    });
    println!("  query_count //item        {:>8.3} ms", t * 1e3);

    let t = fastest(20, || {
        let _ = black_box(oxml_wasm::core::query_text(
            black_box(&parsed),
            "//name",
            &no_ns,
        ));
    });
    println!("  query_text //name         {:>8.3} ms", t * 1e3);

    // The shape a caller in a loop actually produces: the same
    // expression, over and over, against a document already parsed.
    let t = fastest(20, || {
        for _ in 0..100 {
            let _ = black_box(oxml_wasm::core::query_count(
                black_box(&parsed),
                "//item[@id]",
                &no_ns,
            ));
        }
    });
    println!("  100x query_count          {:>8.3} ms", t * 1e3);
}
