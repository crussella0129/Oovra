# Sprint s4 — Build Plan

**Finalized — DO NOT EDIT** (2026-05-20)

Sprint goal: ship the **Compare** view in `oovra-gui` (third tab),
a **Bump patch** button in the editor, and a CLI mirror
`oovra bump-version` so the same capability is available from the
terminal. CLI-first per [`/CLAUDE.md`](../../../CLAUDE.md).

Source: [`../sprint-research/research-report.md`](../sprint-research/research-report.md).

## 1. Sub-schema (elementary)

```
s4 = "diff view + semver bump (lib + CLI + GUI)"
│
├── L. Library (lands first)
│     L4.1  Add `pub fn bump_version(v: &str, kind: BumpKind)
│            -> Result<String>` to `src/header.rs` (next to
│            `is_valid_semver`). `BumpKind` is a small enum
│            { Patch, Minor, Major }.
│     L4.2  Strip pre-release / build-metadata on bump; return
│            the bumped semver string.
│     L4.3  Unit tests for each kind, plus the bad-input case.
│
├── C. CLI (lands second)
│     C4.1  Add `Command::BumpVersion(BumpVersionArgs)` to
│            `src/main.rs`.
│     C4.2  BumpVersionArgs { path: PathBuf, bump: BumpFlag }
│            where BumpFlag is a clap `ValueEnum` mirroring
│            BumpKind.
│     C4.3  run_bump_version(): parse_file -> bump header.version
│            -> oovra::write -> print summary
│            `Bumped <id>: <old> -> <new> at <path>`.
│     C4.4  Integration test in `tests/end_to_end.rs` round-tripping
│            a temp atom through bump-version (library-level via
│            `header::bump_version` + parse/write).
│
├── G. GUI (lands last)
│     G4.1  Extend `CentralView` enum with a third variant
│            `Compare`. Add the tab to the row.
│     G4.2  Add `CompareState` (gui/src/compare.rs):
│            - `a: Option<String>` / `b: Option<String>` ids.
│            - `report: Option<Result<DiffReport, String>>` —
│              computed when both A and B are set, recomputed on
│              change.
│            Methods: `set_a`, `set_b`, `recompute(&lib)`.
│     G4.3  Render the Compare view in `app.rs::render_compare`:
│            - Two ComboBox pickers populated from `self.atom_index`
│              (i.e. anything in the loaded olib).
│            - Below the pickers: render the DiffReport using the
│              same shape as `run_compare` from main.rs but with
│              egui widgets (RichText for color, Grid for the
│              field-changes table, selectable_label for diff
│              hunks).
│            - Empty state: hint "Pick two components to compare."
│            - Same-element-both-sides: hint "Pick a second one."
│            - Mixed-kind: render the error inline.
│     G4.4  Add a `Bump patch` button to `render_editor`'s button
│            row (after Save / Reload). On click, call
│            `header::bump_version(editor.version, BumpKind::Patch)`
│            and assign the result into `editor.version` + set
│            `editor.dirty = true`. User reviews then hits Save.
│
└── T. Test + verify
      T4.1  cargo test -p oovra (library bump_version + CLI integ).
      T4.2  cargo test -p oovra-gui (compare smoke + carry).
      T4.3  cargo build --target wasm32-unknown-unknown -p oovra-gui.
      T4.4  oovra bump-version CLI smoke against a demo atom.
      T4.5  cargo run -p oovra-gui (background); visual heartbeat
            of the Compare tab + Bump button.
      T4.6  Ubuntu/WSL build sanity (cargo build -p oovra under
            WSL with CARGO_TARGET_DIR=/tmp/oovra-linux-target).
```

## 2. Execution sequence

1. L4.1–L4.3 — `header::bump_version` + `BumpKind` enum + unit
   tests.
2. C4.1–C4.3 — clap subcommand + `run_bump_version` in main.rs.
3. C4.4 — integration test for the library bump round-trip.
4. T4.1 — `cargo test -p oovra` confirms tests green.
5. T4.4 — manual smoke of `oovra bump-version` against a demo atom
   (use the Downloads/oovra-mock-library tree).
6. G4.1 — `CentralView::Compare` variant; tab row.
7. G4.2 — `gui/src/compare.rs` with `CompareState` + tests.
8. G4.3 — `render_compare` in app.rs.
9. G4.4 — Bump patch button in render_editor.
10. T4.2, T4.3 — gui tests + wasm32 build.
11. Kill running gui; T4.5 visual heartbeat.
12. T4.6 — WSL Linux build sanity (CLI is the load-bearing piece;
    GUI needs apt prereqs, defer).
13. Sprint-tests docs + agent-tasks + commit.

## 3. Execution details

- **`semver::Version` bump.** Construct `semver::Version::parse(v)`,
  then bump:
  - Patch: `ver.patch += 1`.
  - Minor: `ver.minor += 1; ver.patch = 0`.
  - Major: `ver.major += 1; ver.minor = 0; ver.patch = 0`.
  Clear `ver.pre` and `ver.build` after the bump. Return
  `ver.to_string()`.
- **Editor button only mutates state.** The Bump patch button
  doesn't save — that's the user's next click. Same two-step
  pattern as how the dirty flag drives the Save button label.
- **Compare view recompute.** On any picker change, recompute the
  diff. Caching is overkill for a workbench tool; small diffs are
  microseconds.
- **GUI diff rendering.**
  - Field-changes: use `egui::Grid::new("compare_fields")` with
    three columns (field / before / after).
  - Body diff: iterate the `body_unified_diff` lines and use
    `egui::Label::new(egui::RichText::new(line).color(GREEN))` for
    `+` lines, `.color(RED)` for `-`, and the default color for
    context/hunk-header lines. Wrap in a `ScrollArea`.
  - Compound diff: four list sections, each with a heading and
    bullet rows. Omit any section whose list is empty.
- **No new dependencies.** `similar` (for the diff string) is
  already present transitively via `oovra::diff`. `semver` is
  already a top-level dep.

## 4. Acceptance criteria for s4

- `header::bump_version` returns the right string for each bump
  kind; rejects invalid input.
- `oovra bump-version <file>` writes a bumped file and a re-parse
  shows the new version.
- GUI Compare tab renders atom-vs-atom diffs with colored body
  hunks and a field-changes table; compound-vs-compound shows the
  four structural-diff axes; mixed shows the error.
- GUI Bump patch button updates the editor's version field, flips
  dirty, and the next Save writes the bumped version to disk.
- All prior tests still pass; wasm32 build clean; WSL CLI build
  + test clean.

## 5. Out of scope (deferred)

- Cross-olib compare (both elements from the same olib for now).
- "Fork as new id" / sibling-versioned files. Future "version
  history" sprint.
- Compound recipe editing (still s5+).
- Three-way diff / merge. Out of project scope unless a real
  workflow demands it.
