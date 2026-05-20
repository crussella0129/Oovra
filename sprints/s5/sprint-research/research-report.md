# Sprint s5 — Research Report

**Date:** 2026-05-20
**Sprint goal:** Land the user-confirmed versioning model — filename
suffix `-v<X>-<Y>-<Z>` carries the version; "Save" overwrites with a
confirm gate; "Save As New Version" forks a sibling. Plus rename
editor fields (Filesystem Name / Component-ID) and turn the Library
Components column into a recursive tree.

## 1. Filename suffix grammar — settled

A file's stem may have a trailing version suffix of the form:

```
<canonical-id>-v<X>-<Y>-<Z>
```

where X, Y, Z are non-negative ASCII-digit groups (so `1`, `12`,
`0` all qualify; `1a`, `1.0`, empty do not). Rules:

- The suffix is detected via the **last** `-v` followed by exactly
  three dash-separated all-digit groups, anchored to end-of-stem.
- If the trailing three groups don't match the pattern, the stem
  is taken as a canonical id with **no** version suffix.
- The detected version string is `X.Y.Z` (dashes → dots).
- Examples:
  - `citation-discipline-v1-0-1` → canonical `citation-discipline`, version `1.0.1`
  - `citation-discipline` → canonical `citation-discipline`, no version
  - `my-vendor-prompt-v2-0-0` → canonical `my-vendor-prompt`, version `2.0.0`
  - `name-with-v8-not-a-version` → canonical `name-with-v8-not-a-version`, no version (last `-v` is followed by `8-not-a-version`, not all-digit groups)
  - `prompt-v1-0` → canonical `prompt-v1-0`, no version (only two parts; falls through)

This stays in `oovra::header` as a pure function next to
`is_kebab_case` and `bump_version`. No dependency on filesystem;
it's a string parser.

## 2. "Save" + "Save As New Version" — design

In the Editor's button row:

- **Save** (with `Are you sure?` confirm modal). Overwrites the
  file in place. Same Filesystem Name, same Component-ID. The
  version field reflects whatever's currently in the editor
  (which is whatever the file says, OR what Bump patch produced
  in-session). Confirm copy: *"Save changes to `<id> v<ver>` in
  place? Any compose recipe pinned to that id @ that version
  will see the new content under an existing pin."*
- **Save As New Version** (no confirm). Bumps the version (patch
  by default; a small dropdown next to the button picks minor /
  major), then writes a sibling file
  `<canonical-id>-v<dashed-version>.md`. The new file's header
  has:
    - `id` = the new full stem (with the version suffix).
    - `name` (the Component-ID display) = the canonical id.
    - `version` = the bumped semver string.
    - `meta` = copied from the editor.
  The original file is untouched. The active library reloads so
  the new sibling appears in the Library Components tree.

The Bump patch button from s4 can stay or fold into Save As New
Version. Keep it for now — it's still useful for "just bump the
field without writing a new file" (e.g. fixing the version
metadata of a file that drifted).

## 3. Editor field renames + auto-parse

| Old label | New label        | Source                                            | Editable? |
|-----------|------------------|---------------------------------------------------|-----------|
| id        | Filesystem Name  | `header.id` (equals the file stem by invariant)   | no (s5)   |
| name      | Component-ID     | `header.name`, auto-filled from parsed canonical  | yes       |
| version   | version          | `header.version`, auto-overridden from filename when the suffix is present | yes |
| meta      | meta             | `header.meta`                                     | yes       |

On load, the editor:
1. Reads the file as before.
2. Calls `parse_filename_version(stem)`. If a version is found:
   - Sets the editor's `version` field to the parsed version
     (the filename is authoritative, per the user-chosen convention).
   - Sets the editor's `name` (Component-ID) to the parsed
     canonical id **if** the existing `name` equals the existing
     id (i.e. it hasn't been customized) — otherwise leaves the
     user's chosen `name` alone.

Filesystem Name stays read-only this sprint — renaming a file is
a separate operation (and would ripple into compose pins). Future
sprint can add an explicit Rename action.

## 4. Library Components → recursive tree

Atoms render as leaf rows; compounds render as `egui::CollapsingHeader`s
that expand to show their `composed_of` inputs, recursively. Each
row has the existing left-side checkbox (canvas inclusion) and
the existing row-body click (open in editor). Higher-order
compounds expand further until atoms.

State:
- `pending_open: Option<bool>` on `OovraApp`. Two buttons in the
  Library Components header — `Expand all` / `Collapse all` —
  set this to `Some(true)` / `Some(false)`. On the next frame,
  every CollapsingHeader passes `.open(self.pending_open)`,
  which forces the requested state. After the frame, the flag is
  cleared so manual toggles work again.

Tree-building uses the live `Library`: `self.loaded.elements.get(id)`
gives the element, and `element.header.composed_of` lists the
inputs (themselves library ids). Recursion is on the data side
(no cycles — compose validates DAG). Render side uses egui's
existing CollapsingHeader; no new dependencies.

## 5. CLI mirror — `oovra fork-version`

```
oovra fork-version <FILE> [--bump patch|minor|major]
```

Reads the file, bumps the version, writes a sibling at
`<dir>/<canonical-id>-v<dashed-new-version>.md`, prints
`Forked <id> v<old> -> <new-id> v<new> at <path>`. CLI parity for
agents.

## 6. Universal compatibility (per user's reminder)

All s5 code is in safe Rust on top of `std`, `serde`, `semver`,
and the existing egui workspace deps. No filesystem-specific or
Windows-specific calls. The CLI side will be tested on Ubuntu via
WSL as before:

```bash
wsl.exe -- bash -lc 'source $HOME/.cargo/env && \
  cd /mnt/c/Users/charl/oovra && \
  CARGO_TARGET_DIR=/tmp/oovra-linux-target cargo test -p oovra'
```

The wasm32 build remains the safety net for the gui crate.

## 7. References

- `src/header.rs` — `is_kebab_case` / `slugify` / `bump_version`
  + this sprint's `parse_filename_version`.
- s2 `editor.rs`, s3 `app.rs::render_atom_list`, s4
  `compare.rs` — the patterns the new code extends.
- egui `CollapsingHeader` for the tree:
  [`docs.rs/egui/latest/egui/containers/struct.CollapsingHeader.html`](https://docs.rs/egui/latest/egui/containers/struct.CollapsingHeader.html).
