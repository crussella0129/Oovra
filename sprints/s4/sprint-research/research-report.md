# Sprint s4 — Research Report

**Date:** 2026-05-20
**Sprint goal:** Add the **Compare** view in `oovra-gui` plus a small
**Bump version** affordance in the editor. Mirror the bump on the CLI
so an agent host has the same capability.

This sprint is feature work on existing infrastructure. No new
external crates needed; everything plugs into surfaces that
already exist.

## 1. Library surface (already present)

- `oovra::diff::compare(a: &PromptElement, b: &PromptElement)
  -> Result<DiffReport>` — the FORWARD-DIFF operator, kind-aware:
  - Both atoms → `DiffReport::Content { field_changes,
    body_unified_diff, bodies_equal }`. The body diff is a
    unified-diff string produced via `similar::TextDiff`.
  - Both compounds → `DiffReport::Structural { added, removed,
    version_changed, moved, recipes_equal }`. Sequence-aware.
  - Mixed → `OovraError::KindMismatch`.
- `oovra::Library::load_with` already loads an olib for picker lists.
- The CLI `oovra compare a.md b.md [--format human|json]` already
  wraps `compare` and prints either pretty colored output or JSON.
  The GUI Compare view will render the same `DiffReport` directly,
  no subprocess.

The only **new** library code is a small `bump_version` helper —
incrementing a semver string. The `semver` crate is already a
dependency.

## 2. Semver bump policy — settled

Three kinds (CLI flag `--bump patch|minor|major`):
- **patch:** `1.2.3 → 1.2.4`. Default; covers small content edits.
- **minor:** `1.2.3 → 1.3.0`. Covers meaningful new behavior added
  to an atom while staying backward-compatible.
- **major:** `1.2.3 → 2.0.0`. Reserved for breaking changes — the
  atom's intent or surface has been replaced.

Pre-release / build-metadata segments are stripped on bump
(matching what `cargo` does internally on semver bumps). If the
input doesn't parse as semver, bump returns an error and the caller
keeps the original.

**In place vs new file.** Sprint s4 bumps **in place** — same file,
new version field. This matches how prompts evolve in practice:
the file IS the atom, and git is the canonical history mechanism
for "what changed when." A future sprint can add a "fork as new
id" operation if real workflows need versioned siblings.

## 3. GUI shapes

### Compare tab (third tab in the central panel)

```
[ Editor ] [ Canvas ] [ Compare ]
─────────────────────────────────────────────
Compare two components from the active olib
  A: [ dropdown of element ids ]
  B: [ dropdown of element ids ]
─────────────────────────────────────────────
<DiffReport rendered>
```

Rendering rules:
- **Content diff (atom vs atom):** field changes as a small table
  (field / before / after), then the body unified diff as a
  selectable label block, with `+` lines colored green and `-`
  lines red (using `egui::RichText::color`).
- **Structural diff (compound vs compound):** four bulleted lists
  for added / removed / version_changed / moved. Empty lists
  collapse to an "(identical)" line.
- **Mixed (atom vs compound):** show the error message inline.
- **Same id picked for both:** show "Same element on both sides;
  pick a second one."

Neither dropdown is required at startup — empty selection just
shows a hint.

### Bump patch button in the editor

In the editor's button row, alongside `Save *` and `Reload`, add a
small `Bump patch` button. Clicking it:
- Parses the current `version` field.
- Bumps the patch (1.0.0 → 1.0.1).
- Updates the editor's version field; flips `dirty = true`.
- The user reviews and hits `Save *` to persist.

This is the two-step pattern: the bump is editorial-intent, the
save is the commit. Keeps the user in control.

## 4. CLI surface — one new subcommand

```
oovra bump-version <FILE> [--bump patch|minor|major]
```

- Parses the file, bumps the header's version field, writes back
  via `oovra::write` (in-memory validation pass guards against
  garbage on disk).
- Default `--bump patch`.
- Prints a one-line summary: `Bumped <id>: <old> → <new> at <path>`.

This is the CLI mirror of the editor button — same operation,
scriptable for agents and CI.

## 5. Scope notes / out of scope

- **Cross-olib diff.** s4's Compare picks both sides from the
  currently loaded olib. Cross-olib comparison (pick A from one,
  B from another) is meaningful but adds picker state we can
  defer; a later sprint can hoist the picker out of the loaded
  library.
- **Compound recipe editing.** Still deferred. The "version
  changed" axis only tells you a compound's input pin moved; it
  doesn't let you rewrite the recipe. That's the same scope as
  s3 deferred.
- **No new external crates.** `similar` is already in for the
  diff output; `semver` is already in for parsing/bumping. The
  egui side uses widgets already present in 0.34.x.

## 6. References

- `src/diff.rs` — `compare`, `DiffReport`, `ContentDiff`,
  `StructuralDiff` (lines 26–90 for the public shapes).
- `src/header.rs` — `is_valid_semver` (lines ~202), drives the
  precondition for bump; the bump itself uses `semver::Version`.
- `src/main.rs` — `run_compare` shows the canonical
  human-rendering pattern; the GUI re-renders the same data with
  egui widgets.
- s3 build-plan / app.rs patterns for adding a tab and per-row
  state.
