# Sprint s5 — Test Report

**Date:** 2026-05-20
**Sprint:** s5 — Filename-suffix versioning + Save / Save As New
Version + recursive Library Components tree.
**Verdict:** **PASS** — sprint s5 complete; all acceptance criteria met.

## 1. Result matrix

| Category    | Tests run | Passed | Failed | Deferred |
|-------------|----------:|-------:|-------:|---------:|
| Unit        | 11 (U5-1…U5-8 + 3 extras) + carries | 11 | 0 | 0 |
| Integration | 7 (I5-1…I5-7)             | 7  | 0 | 1 (CI) |
| End-to-end  | 2 (E5-1, E5-2)            | 2  | 0 | 0 |

Detailed records: [`unit-tests.md`](./unit-tests.md),
[`integration-tests.md`](./integration-tests.md),
[`e2e-tests.md`](./e2e-tests.md).

## 2. Acceptance criteria (from build-plan.md §4)

- [x] `parse_filename_version` correct for the table in the
      research report (U5-1…U5-6 + extras).
- [x] `oovra fork-version <file>` writes the expected sibling and
      prints a summary (I5-1, I5-6).
- [x] GUI editor shows **Filesystem Name** (read-only) and
      **Component-ID** (editable). For versioned files, the
      version field reflects the parsed filename version
      (U5-8, E5-1).
- [x] Save button confirms before overwriting; Save As New
      Version writes a sibling and the library reloads (E5-1).
- [x] Library Components is a tree; compounds expand to show
      their inputs; Expand all / Collapse all behave (E5-1).
- [x] All prior tests still pass on Windows and on Ubuntu via
      WSL; wasm32 clean.

## 3. Cross-platform check

Both the new filename-parser logic and the fork-version CLI run
identically on Windows and on Ubuntu Linux via WSL. WSL cargo test
reports **81 passing** — same numbers, same suite — confirming
that no s5 code introduced Windows-only assumptions. The Linux
build runs against the same source files with
`CARGO_TARGET_DIR=/tmp/oovra-linux-target` so artifacts don't
clobber the Windows `target/`.

The GUI's tree view, save-confirm Window, and ComboBox dropdown
are pure egui — no platform-specific calls. The wasm32 build
proves the GUI crate compiles for the web target; the WSL GUI
build would need apt prereqs (deferred as before).

## 4. Issues found and root-caused this sprint

### Issue 1 — `collapsible_if` clippy warning in the atom row

**Symptom:** clippy flagged the click + `if let` pair in
`render_tree_node` as a candidate for the `if-let-chain` form.

**Fix:** rewrote as
`if response.clicked() && let Some(i) = index_pos { … }` per
clippy's own suggestion. The chained form is stable in Rust 1.95
(our toolchain on both Windows and WSL).

**Disposition:** workspace clippy now clean.

## 5. Deferred items

- CI / `gh` push verification.
- Cross-olib compare (still deferred from s4).
- Explicit Rename action (changing Filesystem Name without
  forking). The Filesystem Name field is read-only this sprint.
- Compound recipe editing in the GUI.
- egui Panel-alias migration.
- GUI Linux build (apt prereqs).

## 6. Sprint close

Sprint s5 is **complete and locked**. The GUI gains:
- Three-tab central panel (Editor / Canvas / Compare).
- Recursive Library Components tree with Expand all /
  Collapse all.
- Save (with confirm) + Save As New Version (with patch/minor/
  major dropdown).
- Auto-parse of `-v<X>-<Y>-<Z>` filename suffix into the
  version field.

The CLI gains `oovra fork-version`. The library gains
`header::parse_filename_version` and `header::compose_versioned_filename`,
plus `BumpKind::default() == Patch`.

Roadmap next pickup is **s6 — egui Panel-alias migration** (the
deprecated `TopBottomPanel` / `SidePanel` aliases) or returning
to the **WASM filesystem shim** (originally s5 on the roadmap;
deferred because the user pivoted to versioning first). The
user can pick.
