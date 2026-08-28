#!/usr/bin/env bash
#
# Everything CI runs, locally, in the order that fails fastest.
#
# This exists because a local check that is a *subset* of CI reports
# success over the difference. In `oxml` that gap was one flag:
# `scripts/gate.sh` did not set `RUSTFLAGS`, CI did, and a build that
# only warned locally turned three jobs red on the pull request.
#
# Run it before pushing. It is slower than `cargo "+$TOOLCHAIN" test`; it is much
# faster than a round-trip through CI.
set -uo pipefail
# Resolve the repository from this script's own location, not from
# the caller's. `git rev-parse` reads the *current* directory, so
# running this by absolute path from anywhere else failed with
# "fatal: not a git repository".
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# The toolchain is named explicitly rather than left to
# `rust-toolchain.toml`. A `RUSTUP_TOOLCHAIN` in the environment
# silently overrides that file, and then the pinned version this repo
# went to the trouble of choosing is not the one running -- which is
# how a clippy lint that exists in 1.98 and not in 1.97 made a green
# local run and a red CI one.
export TOOLCHAIN="${OXML_WASM_TOOLCHAIN:-1.98.0}"
MSRV="${OXML_WASM_MSRV:-1.86.0}"
# CI sets this for every job, so a warning there is a failure. Setting
# it here is what makes this script predict CI rather than approximate
# it.
export RUSTFLAGS="${RUSTFLAGS:--D warnings}"
FAILED=()
LOG="$(mktemp -t oxml-wasm-gate)"
trap 'rm -f "$LOG"' EXIT

step() {
  local name="$1"; shift
  printf '%-34s' "$name"
  if "$@" > "$LOG" 2>&1; then
    echo "ok"
  else
    echo "FAIL"
    FAILED+=("$name")
    tail -25 "$LOG" | sed 's/^/    /'
  fi
}

# A local per-project target-dir scheme puts a symlink at `target`, and
# a `git add -A` will commit it. `.gitignore` does not help once a path
# is tracked, and CI then cannot create its build directory on top of
# it -- every build job fails until it is removed.
step "no tracked build dir" bash -c '! git ls-files | grep -qx target'
step "fmt"            cargo "+$TOOLCHAIN" fmt --all --check
step "clippy"         cargo "+$TOOLCHAIN" clippy --all-targets --all-features -- -D warnings
step "tests"          cargo "+$TOOLCHAIN" test --all-features
step "rustdoc"        env RUSTDOCFLAGS="-D warnings" cargo "+$TOOLCHAIN" doc --no-deps --all-features
# CI greps for this rather than trusting that it is still there. A
# crate that quietly loses the attribute keeps compiling.
step "forbid unsafe" bash -c '
  grep -rq "#!\[forbid(unsafe_code)\]" src/ || {
    echo "#![forbid(unsafe_code)] is missing"; exit 1; }'
# A change that builds natively can still fail to compile to wasm, and
# that failure would otherwise surface only to whoever ran `wasm-pack`
# next.
step "wasm build"     cargo "+$TOOLCHAIN" build --target wasm32-unknown-unknown --release

# `wasm-pack` is slow and not everyone has it. Skip loudly rather than
# silently: a step that vanishes when a tool is missing is a step that
# reports success over what it did not run.
if command -v wasm-pack >/dev/null 2>&1; then
  step "wasm-pack build" env RUSTUP_TOOLCHAIN="$TOOLCHAIN" wasm-pack build --target web --release
  # The `#[wasm_bindgen]` layer cannot run under `cargo "+$TOOLCHAIN" test`, so
  # without this nothing exercises the JavaScript-facing API at all.
  step "wasm-pack test" env RUSTUP_TOOLCHAIN="$TOOLCHAIN" wasm-pack test --node
  step "bindings generated" bash -c '
    test -f pkg/oxml_wasm_bg.wasm &&
    test -f pkg/oxml_wasm.js &&
    test -f pkg/oxml_wasm.d.ts &&
    grep -q queryText pkg/oxml_wasm.d.ts'
  # These need the nodejs target, which `run-all.sh` builds itself.
  step "node examples"  env RUSTUP_TOOLCHAIN="$TOOLCHAIN" ./examples/run-all.sh
else
  printf '%-34s%s\n' "wasm-pack steps" "SKIPPED (wasm-pack not installed)"
  FAILED+=("wasm-pack steps not run")
fi

step "coverage 95%" cargo "+$TOOLCHAIN" llvm-cov --all-features --fail-under-lines 95 \
  --ignore-filename-regex 'builds/cargo/package|src/lib\.rs'
step "MSRV $MSRV" cargo "+$MSRV" check --all-features

echo
if [ "${#FAILED[@]}" -eq 0 ]; then
  echo "all green"
  exit 0
fi
printf 'FAILED: %s\n' "${FAILED[@]}"
exit 1
