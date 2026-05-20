# Sprint s5 — Test Plan

**Finalized — DO NOT EDIT** (2026-05-20)

## 1. Unit tests

### Library — `oovra::header`

- **U5-1** `parse_filename_version_strips_three_part_suffix`:
  `citation-discipline-v1-0-1` → `("citation-discipline", Some("1.0.1"))`.
- **U5-2** `parse_filename_version_no_suffix`:
  `citation-discipline` → `("citation-discipline", None)`.
- **U5-3** `parse_filename_version_handles_v_in_canonical`:
  `my-vendor-prompt-v2-0-0` → `("my-vendor-prompt", Some("2.0.0"))`.
- **U5-4** `parse_filename_version_rejects_two_part_suffix`:
  `prompt-v1-0` → `("prompt-v1-0", None)` (not 3 parts).
- **U5-5** `parse_filename_version_rejects_non_digit_after_v`:
  `name-with-v8-not-a-version` → unchanged (the trailing groups
  aren't all-digit).
- **U5-6** `parse_filename_version_handles_multidigit_parts`:
  `prompt-v12-34-56` → `("prompt", Some("12.34.56"))`.
- **U5-7** `compose_versioned_filename_inverse`:
  building `<canonical>-v<dashed>` from `("citation-discipline",
  "1.0.1")` yields `citation-discipline-v1-0-1`.

### GUI — `oovra-gui`

- **U5-8** `editor_autoparse_overrides_version_when_filename_suffixed`:
  load an atom whose filename has `-v1-0-1` and whose header
  version is `1.0.0`; assert the editor's version field reads
  `1.0.1` after `Editor::open`.
- **U5-9** `editor_save_as_new_version_writes_sibling`:
  open an atom; call the "save-as-new-version" code path with
  Patch; assert a sibling file exists at the computed path with
  the bumped version and the canonical id preserved in
  `header.name`.

## 2. Integration tests

- **I5-1** `fork_version_creates_versioned_sibling` (in
  `tests/end_to_end.rs`): label an atom 1.0.0, fork via the
  library-level path (mirroring what the CLI does), parse the
  sibling, assert filename and header fields are correct.
- **I5-2** `cargo test -p oovra` total target: 70 + 6 unit + 1
  integ = **77 PASS**.
- **I5-3** `cargo test -p oovra-gui` target: 14 + 2 = **16 PASS**.
- **I5-4** `cargo build --target wasm32-unknown-unknown -p oovra-gui`
  PASS.
- **I5-5** workspace clippy clean.
- **I5-6** `oovra fork-version` CLI smoke against the mock library.
- **I5-7** WSL Ubuntu cargo test of oovra (cross-platform sanity).

## 3. End-to-End

- **E5-1** Visual heartbeat. With the new tree view, the user:
  1. Opens `C:\Users\charl\Downloads\oovra-mock-library`.
  2. Selects `coding-agent/olib`. The Library Components column
     shows four `· ` atoms and a `▣ coding-agent-prompt` row
     with a triangle.
  3. Clicks the triangle → the compound expands to show its
     four atom inputs nested under it. Each input has a
     checkbox + clickable row.
  4. Clicks "Expand all" → every collapsible opens. Clicks
     "Collapse all" → every collapsible closes.
  5. Opens `tone-direct` in the editor. The editor shows
     **Filesystem Name** `tone-direct`, **Component-ID**
     `tone-direct`, **version** `1.0.1` (left at that from the
     s4 smoke).
  6. Clicks **Save As New Version** (Patch). A new sibling
     `tone-direct-v1-0-2.md` appears in Library Components with
     the canonical id `tone-direct` preserved. Opens it; the
     Filesystem Name reads `tone-direct-v1-0-2`, the
     Component-ID reads `tone-direct`, version reads `1.0.2`.
  7. Hits **Save** on an atom → confirm dialog appears
     summarizing what's about to happen; Yes proceeds.

## 4. CI — DEFERRED (same posture as prior sprints)
