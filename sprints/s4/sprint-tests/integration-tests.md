# Sprint s4 — Integration Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §2.

## I4-1 — `cargo test -p oovra`

70 tests pass on Windows: 41 + 4 + 25. New since s3:

- 5 new lib unit tests for `bump_version` (U4-1…U4-5).
- 1 new integration test in `tests/end_to_end.rs`:
  `bump_version_round_trips_an_atom`. Writes an atom at 1.2.3,
  bumps via `header::bump_version` + `oovra::write`, re-parses and
  asserts the file now reports 1.2.4 with the id unchanged.

**PASS.**

## I4-2 — `cargo test -p oovra-gui`

14 tests pass: 3 app + 3 editor + 4 canvas + 4 compare. The compare
suite covers the empty-pick / content / mixed / same-id paths
(U4-6…U4-8 + the same-id-both-sides clearing).

**PASS.**

## I4-3 — `cargo build --target wasm32-unknown-unknown -p oovra-gui`

13.25s, exit 0. The compare module compiles cleanly for wasm32 —
no `std::fs` references in its body; the unit tests are
`#[cfg(test)]` so they don't enter the wasm32 build.

**PASS.**

## I4-4 — workspace clippy

`cargo clippy --workspace --all-targets` exit 0, no new warnings.

**PASS.**

## I4-5 — `oovra bump-version` CLI smoke

Against the mock library:

```
$ oovra inspect ./tone-direct.md | grep version
  version  1.0.0

$ oovra bump-version ./tone-direct.md
Bumped tone-direct: 1.0.0 -> 1.0.1 at ./tone-direct.md

$ oovra inspect ./tone-direct.md | grep version
  version  1.0.1
```

`--bump minor` and `--bump major` work via the same code path
(library unit tests U4-2 / U4-3 cover them); manual CLI smoke
covered patch end-to-end.

**PASS.**

## I4-6 — Ubuntu Linux build + test via WSL

```
$ wsl.exe -- bash -lc 'source $HOME/.cargo/env && \
    cd /mnt/c/Users/charl/oovra && \
    CARGO_TARGET_DIR=/tmp/oovra-linux-target cargo test -p oovra'
running 25 tests ... ok
test result: ok. 25 passed; 0 failed
```

Lib unit + main unit + 25 integration tests all green on Ubuntu,
exit 0. The cross-platform invariant from CLAUDE.md holds — no
sprint-s4 code introduces Windows-only assumptions.

**PASS.**

## CI verification — DEFERRED (same posture as prior sprints)

## Summary

| ID  | Test                                          | Status |
|-----|-----------------------------------------------|--------|
| I4-1 | `cargo test -p oovra` (70)                    | PASS |
| I4-2 | `cargo test -p oovra-gui` (14)                | PASS |
| I4-3 | `cargo build wasm32 -p oovra-gui`             | PASS |
| I4-4 | workspace clippy clean                        | PASS |
| I4-5 | `oovra bump-version` CLI smoke                | PASS |
| I4-6 | Ubuntu Linux build + test via WSL             | PASS |
| CI  | GitHub Actions verification                    | DEFERRED |
