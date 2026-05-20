# Sprint s4 — Test Plan

**Finalized — DO NOT EDIT** (2026-05-20)

## 1. Unit tests

### Library — `oovra::header`

- **U4-1 — `bump_version` patch.** `bump_version("1.2.3", Patch)`
  returns `"1.2.4"`.
- **U4-2 — `bump_version` minor.** `bump_version("1.2.3", Minor)`
  returns `"1.3.0"`.
- **U4-3 — `bump_version` major.** `bump_version("1.2.3", Major)`
  returns `"2.0.0"`.
- **U4-4 — pre-release / build-metadata stripped.**
  `bump_version("1.2.3-rc1+sha", Patch)` returns `"1.2.4"`.
- **U4-5 — bad input rejected.** `bump_version("not-a-version", Patch)`
  returns `Err`.

### GUI — `oovra-gui::compare`

- **U4-6 — `CompareState::recompute` empty.** When either pick is
  None, `recompute` clears `report` (no diff shown).
- **U4-7 — `CompareState::recompute` happy path.** Build a tiny
  library with two atoms differing only in body; set A and B;
  recompute; assert report is `Ok(DiffReport::Content { ... })`
  with `bodies_equal == false`.
- **U4-8 — `CompareState::recompute` mixed kind.** Same-side an
  atom and a compound; assert report is `Err(...)` matching the
  `KindMismatch` error path.

## 2. Integration tests

- **I4-1 — `cargo test -p oovra` still PASS.** Adds U4-1…U4-5
  (5 new lib unit tests) + a new integration test
  `bump_version_round_trips_an_atom` that bumps a real atom on
  disk and re-parses. Total expected: 64 + 5 unit + 1 integ = 70.
- **I4-2 — `cargo test -p oovra-gui` PASS.** Adds U4-6…U4-8
  (3 new compare-state tests) on top of s3's 10. Target: 13.
- **I4-3 — `cargo build wasm32 -p oovra-gui` PASS.**
- **I4-4 — workspace clippy clean.**
- **I4-5 — `oovra bump-version` CLI smoke.** Bump an atom in the
  mock library, verify the summary line and that the file now
  shows the new version (via `oovra inspect`).
- **I4-6 — WSL Ubuntu build sanity.** `cargo build -p oovra` via
  WSL with `CARGO_TARGET_DIR=/tmp/oovra-linux-target` PASS.

## 3. End-to-End test

- **E4-1 — Compare tab + Bump button heartbeat.**
  `cargo run -p oovra-gui`. User walkthrough:
  1. Open `C:\Users\charl\Downloads\oovra-mock-library`.
  2. Select `coding-agent/olib`.
  3. Switch to the **Compare** tab. Pick `role-declaration` for
     A and `tone-direct` for B. The diff renders a field-changes
     row (id/name/meta differ) and a colored body diff.
  4. Same-side both → "Pick a second one." Mixed (atom vs the
     `coding-agent-prompt` compound) → KindMismatch error inline.
  5. Switch back to **Editor**. Click `role-declaration`. Click
     **Bump patch**. The version field changes `1.0.0 → 1.0.1`
     and Save flips to `Save *`. Click Save. `oovra inspect`
     from a terminal confirms the new version on disk.

## 4. CI verification — DEFERRED (same posture as prior sprints)

## 5. Logging conventions — same as prior sprints
