# Sprint s5 — Build Plan

**Finalized — DO NOT EDIT** (2026-05-20)

Sprint goal: filename-suffix versioning + Save / Save As New Version
+ editor relabels + recursive tree-view of Library Components.
CLI-first per [`/CLAUDE.md`](../../../CLAUDE.md).

Source: [`../sprint-research/research-report.md`](../sprint-research/research-report.md).

## 1. Sub-schema (elementary)

```
s5 = "filename versioning + Save/SaveAsNewVersion + tree view"
│
├── L. Library
│     L5.1  header::parse_filename_version(stem) -> (canonical, Option<String>).
│            Last-occurrence -v<X>-<Y>-<Z> rule, all-ASCII-digit groups.
│            Six+ unit tests covering the table in the research report.
│     L5.2  header::compose_versioned_filename(canonical, version) -> String,
│            the inverse for Save As New Version filename construction.
│
├── C. CLI
│     C5.1  Command::ForkVersion(ForkVersionArgs) in main.rs.
│     C5.2  run_fork_version: parse_file -> bump version ->
│            build new filename via compose_versioned_filename ->
│            write at <dir>/<new-stem>.md -> print summary.
│     C5.3  Integration test in tests/end_to_end.rs:
│            fork_version_creates_versioned_sibling.
│
├── G. GUI
│     G5.1  Editor field relabels in app.rs::render_editor: "id" ->
│            "Filesystem Name" (read-only label), "name" -> "Component-ID"
│            (still editable). Render uses ui.add_enabled(false, ...) for the
│            Filesystem Name TextEdit so it visually reads as read-only.
│     G5.2  Editor::open auto-fills: if filename has suffix, overwrite
│            ed.version with parsed value; if header.name == header.id
│            (uncustomized default), set ed.name to the parsed canonical.
│     G5.3  Editor button row split: Save (with confirm dialog) +
│            Save As New Version + Bump dropdown.
│            Save confirm: use a small egui Window or just an
│            "are you sure?" inline checkbox flag.
│            Save As New Version: produces the sibling file at
│            <olib>/<canonical>-v<dashed>.md.
│     G5.4  Library Components becomes a tree:
│            - new gui/src/tree.rs with `render_tree(ui, app_state)` that
│              walks library elements; atoms = leaf row, compounds =
│              CollapsingHeader containing recursive nodes.
│            - Each node keeps the checkbox-toggle-canvas + click-to-edit
│              two-click-target pattern from s2/s3.
│            - Add a header row above the tree: "Expand all" and "Collapse
//              all" buttons that set OovraApp::pending_open.
│            - When pending_open is Some(b), every CollapsingHeader passes
//              .open(Some(b)) once, then the flag clears.
│     G5.5  Helper on OovraApp: save_in_place + save_as_new_version that
│            both go through oovra::write. save_as_new_version uses the
│            library's compose_versioned_filename to derive the path.
│
└── T. Test + verify
      T5.1  cargo test -p oovra (lib unit + CLI integ tests).
      T5.2  cargo test -p oovra-gui (any new tree/editor logic tests).
      T5.3  cargo build --target wasm32-unknown-unknown -p oovra-gui.
      T5.4  CLI smoke for fork-version against the mock library.
      T5.5  Compose a real compound in the mock library so the tree
            has interesting content to demonstrate.
      T5.6  Visual heartbeat: tree expansion + Save As New Version.
      T5.7  WSL Ubuntu cargo test -p oovra (cross-platform sanity).
```

## 2. Execution sequence

1. L5.1 + tests.
2. L5.2 (inverse).
3. T5.1 partial: cargo test -p oovra.
4. C5.1–C5.2 fork-version subcommand.
5. C5.3 integ test.
6. T5.1 again.
7. G5.1 / G5.2 (editor relabels + auto-parse).
8. G5.3 (Save / Save As New Version / confirm dialog).
9. G5.4 (tree view + expand/collapse).
10. T5.2, T5.3.
11. T5.4 CLI smoke against mock library.
12. T5.5 compose a real compound.
13. Kill gui; T5.6 visual heartbeat.
14. T5.7 WSL sanity.
15. Sprint-tests docs + commit.

## 3. Execution details

- **`parse_filename_version` is purely a string parser.** No I/O,
  no semver crate dependency on the input format itself — we
  validate digit groups directly and convert dashes to dots; the
  result string is what gets handed to `semver::Version::parse`
  if needed elsewhere.
- **Save confirm dialog** in egui can be done via a `Window`
  conditionally shown when `app.save_confirm_pending == true`,
  with Yes / No buttons. Keeps the flow keyboard-friendly: click
  Save -> dialog appears -> Yes / No.
- **Auto-parse on Editor::open** mutates only the in-memory
  state; the on-disk file is unchanged until the user saves. So
  loading a file whose filename suffix disagrees with its
  header.version surfaces in the UI (the editor shows the
  filename-derived version); a subsequent Save persists the
  reconciliation.
- **Tree rendering recursion** walks
  `self.loaded.elements[id].header.composed_of`. The library is
  already validated as a DAG by compose (cycles would have been
  rejected at compose time), so no cycle detection needed in the
  renderer. We still cap nesting at depth 16 as a defensive
  belt-and-suspenders bound (way past any real recipe).
- **Borrow-checker discipline.** The tree render function takes
  `&mut self` and recurses. For each node, snapshot the input
  ids (`Vec<String>`) before opening the CollapsingHeader closure
  so we can iterate without holding a self.loaded borrow during
  recursion.
- **Expand/Collapse all.** `pending_open: Option<bool>` flag on
  OovraApp. Set by the two header buttons; consumed once per
  frame inside `render_tree` after the walk.

## 4. Acceptance criteria

- `parse_filename_version` returns the expected (canonical,
  version) pairs for the table in the research report.
- `oovra fork-version <file>` writes a new sibling at the
  computed path and prints the summary.
- GUI editor shows "Filesystem Name" (read-only) and "Component-ID"
  (editable). For versioned files, the editor's version field
  reflects the parsed filename version.
- Save button shows a confirm dialog; Save As New Version writes
  a sibling and the library reloads to surface it.
- Library Components is a tree; compounds expand to show their
  inputs; Expand all / Collapse all set every node at once.
- All existing tests still pass on Windows and on Ubuntu via WSL.
- `cargo build wasm32 -p oovra-gui` still clean.

## 5. Out of scope

- Rename action (changing the Filesystem Name without forking).
- Cross-olib compound expansion (the tree resolves against the
  loaded olib only; missing ids surface as "(missing: <id>)").
- Persisted expand/collapse state per session.
