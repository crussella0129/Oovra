# Sprint s4 — Test Report

**Date:** 2026-05-20
**Sprint:** s4 — Diff / versioning view + `oovra bump-version`
**Verdict:** **PASS** — sprint s4 complete; all acceptance criteria met.

## 1. Result matrix

| Category    | Tests run | Passed | Failed | Deferred |
|-------------|----------:|-------:|-------:|---------:|
| Unit        | 9 (U4-1…U4-8 + same-id) + carries | 9 | 0 | 0 |
| Integration | 6 (I4-1…I4-6)             | 6  | 0 | 1 (CI) |
| End-to-end  | 2 (E4-1, E4-2)            | 2  | 0 | 0 |

Detailed records: [`unit-tests.md`](./unit-tests.md),
[`integration-tests.md`](./integration-tests.md),
[`e2e-tests.md`](./e2e-tests.md).

## 2. Acceptance criteria (from build-plan.md §4)

- [x] `header::bump_version` returns the right string for each
      bump kind and rejects invalid input (U4-1…U4-5).
- [x] `oovra bump-version <file>` writes a bumped file; re-parse
      shows the new version (I4-5, integration test in `tests/`).
- [x] GUI Compare tab renders atom-vs-atom diffs with colored
      hunks and a field-changes table; compound-vs-compound shows
      the four structural-diff axes; mixed-kind shows the error
      (E4-1, U4-7, U4-8).
- [x] GUI Bump patch button updates the editor's version field,
      flips dirty, and Save writes the bumped version to disk
      (E4-1).
- [x] 70 oovra tests + 14 gui tests + wasm32 build still clean +
      WSL Ubuntu build/test still PASS.

## 3. Issues found and root-caused this sprint

No new issues. The carry-forward items (egui Panel-alias
deprecation, GUI Linux build apt-prereqs, CI push verification)
remain on the roadmap; this sprint added nothing to that list.

## 4. Deferred items

- **CI / `gh` push verification** — same posture as prior sprints.
- **Cross-olib compare.** Both sides currently picked from the
  loaded olib. A later sprint can lift the picker out of the
  loaded library.
- **Fork as new id / sibling-versioned files.** A different
  versioning model where each bump creates a new file with a
  version-suffixed id. The current s4 model is in-place bump +
  git for history; sibling-versioning is a future workflow we
  can add when a real need surfaces.
- **Compound recipe editing.** Still deferred.
- **egui Panel-alias migration.** Still tracked on the roadmap.

## 5. Sprint close

Sprint s4 is **complete and locked**. The GUI now has three tabs:
Editor / Canvas / Compare. The CLI has a new `bump-version`
subcommand. The library's `header::bump_version` helper is the
single source of truth for semver bumps and is exercised by both
the CLI and the GUI button.

Next per the roadmap: **s5 — WASM filesystem shim + Trunk
pipeline** (the web build's missing filesystem layer) or **s6 —
egui Panel-alias migration** if you'd rather clear the deprecation
warnings before adding more features. s5 is the higher-leverage
pickup.
