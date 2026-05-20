# Sprint s5 — Integration Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §2.

## I5-1 — `fork_version_creates_versioned_sibling`

New integration test in `tests/end_to_end.rs`. Writes a v1.0.0 atom,
runs the same parse / bump / compose-filename / write sequence that
`oovra fork-version` performs, then re-parses to assert:

- The new file at `<olib>/<canonical>-v1-0-1.md` exists.
- Its `header.id` matches the new stem (`atom-v1-0-1`).
- Its `header.name` preserves the canonical (`atom`).
- Its `header.version` is the bumped value (`1.0.1`).
- The original file at `<olib>/atom.md` is untouched (still v1.0.0).

**PASS.**

## I5-2 — `cargo test -p oovra`

81 tests pass on Windows: 51 lib unit + 4 main unit + 26 integ.

**PASS.**

## I5-3 — `cargo test -p oovra-gui`

15 tests pass: 3 app + 4 editor + 4 canvas + 4 compare.

**PASS.**

## I5-4 — `cargo build --target wasm32-unknown-unknown -p oovra-gui`

11.47s, exit 0. The new tree-view and dialog code compiles cleanly
for wasm32 (no filesystem references in the shared code paths).

**PASS.**

## I5-5 — workspace clippy clean

`cargo clippy --workspace --all-targets` exit 0. The earlier
`collapsible_if` warning in the tree-view click handler was fixed
by adopting the `if expr && let Some(_) = …` chain form (stable
in our Rust 1.95).

**PASS.**

## I5-6 — `oovra fork-version` CLI smoke

Against the mock library:

```
$ oovra fork-version C:/Users/charl/Downloads/oovra-mock-library/research-flow/olib/citation-discipline.md
Forked citation-discipline v1.0.0 -> citation-discipline-v1-0-1 v1.0.1
  at C:/Users/charl/Downloads/oovra-mock-library/research-flow/olib\citation-discipline-v1-0-1.md

$ oovra inspect <new-sibling>
  id        citation-discipline-v1-0-1
  name      citation-discipline           # canonical preserved
  version   1.0.1                          # bumped
```

The original file remains at v1.0.0. **PASS.**

## I5-7 — WSL Ubuntu cargo test of oovra

```
$ wsl.exe -- bash -lc 'source $HOME/.cargo/env && \
    cd /mnt/c/Users/charl/oovra && \
    CARGO_TARGET_DIR=/tmp/oovra-linux-target cargo test -p oovra'

test result: ok. 51 passed                  # lib unit
test result: ok. 4 passed                   # main unit
test result: ok. 26 passed                  # integration
```

**81 PASS on Ubuntu Linux**, identical to Windows. The
filename-suffix parsing, fork-version CLI, and bump_version logic
are all pure Rust + std + semver and have no platform-specific
code paths. CLAUDE.md cross-platform invariant holds.

**PASS.**

## CI verification — DEFERRED (same posture as prior sprints)

## Summary

| ID  | Test                                          | Status |
|-----|-----------------------------------------------|--------|
| I5-1 | `fork_version_creates_versioned_sibling`     | PASS |
| I5-2 | `cargo test -p oovra` (81)                   | PASS |
| I5-3 | `cargo test -p oovra-gui` (15)               | PASS |
| I5-4 | `cargo build wasm32 -p oovra-gui`            | PASS |
| I5-5 | workspace clippy clean                       | PASS |
| I5-6 | `oovra fork-version` CLI smoke                | PASS |
| I5-7 | WSL Ubuntu cargo test of oovra (81 PASS)     | PASS |
| CI  | GitHub Actions verification                    | DEFERRED |
