# Changelog

All notable changes to Oovra are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **`oovra create` is now file-based, with two modes.** `--new` (CLI-arg
  scaffolding) is removed; authoring is always from plain Markdown files
  written in an editor, which sidesteps shell-quoting of prompt content
  entirely.
  - `oovra create --label <path>...` headers files **in place** — each
    file becomes the element.
  - `oovra create --olib <path>...` writes headered **copies** into the
    `olib/` library, leaving the originals untouched.
  Both accept files and directories (a directory contributes the `.md`
  files directly inside it) and print a `written / skipped / failed`
  summary over the batch.
- **The default library directory is now `./olib`** (was `./elements`)
  for `oovra create` and `oovra compose`; `--library` still overrides it.

### Added

- **olib-to-olib transfer.** An input to `--olib` that is already an
  Oovra file is copied **verbatim** — no second header — so `--olib`
  doubles as a way to move elements between libraries.
- **`slugify` + interactive id resolution.** The element id is the
  filename stem; a non-kebab-case stem is slugified, then confirmed
  interactively (use the slug / rename the file / skip). `--slug`
  auto-applies the slug with no prompt; non-interactive runs skip
  non-kebab-named files with advice.

### Removed

- `oovra create --new` and the `~~~` inline-body marker — superseded by
  the file-based `--label` / `--olib` modes.

## [0.2.0] — 2026-05-13

This release replaces the numeric `order` field with an explicit
`kind = "atom" | "compound"` discriminator, plus a handful of related
cleanups. It is a **breaking schema change**: every existing Oovra file
needs to be migrated. The `oovra migrate` subcommand does this in place.

### Added

- **`kind` discriminator on every prompt element.** Two legal values:
  `"atom"` (hand-authored, no recipe) and `"compound"` (produced by
  `oovra compose`). Required on every file.
- **`depth` field on compounds** (optional, written by `compose`).
  Computed as `1 + max(child.depth, atoms = 0)`, identical numerically
  to `body_level` but exposed for human-readable filtering and tooling.
- **`oovra migrate <library>` subcommand** that walks a library
  recursively and rewrites v0.1 files (with `order`) to v0.2 (with
  `kind`). Recursively rewrites embedded frontmatters inside compound
  bodies; preserves `generated_at` verbatim.
- **`--legacy` global flag** for read-only ergonomics during the
  transition. Files with the v0.1 `order` schema parse correctly under
  `--legacy`; writes are always in v0.2 format. Removed in v0.3.
- **Sequence-aware `compare`.** Structural diff now reports a `moved`
  list when inputs are reordered in `composed_of`. Reordering changes
  the rendered prompt, so v0.1's order-blind diff was lying about
  equivalence.
- New error variants: `KindMismatch`, `CannotDecomposeAtom`,
  `AtomHasForbiddenField`, `CompoundMissingField`.
- New `ParseOptions` / `parse_with` / `parse_file_with` /
  `Library::load_with` for library consumers needing explicit control
  over legacy mode.
- Dual MIT / Apache-2.0 license files (`LICENSE-MIT`, `LICENSE-APACHE`)
  matching the `Cargo.toml` declaration. The repo previously shipped a
  GPL-3.0 `LICENSE` file in three-way conflict with the README and the
  Cargo.toml.

### Changed

- **`StructuralDiff` wire shape**: `added` and `removed` are now
  `Vec<PositionedInput>` carrying the input's position in
  `composed_of`. JSON output reflects this. `order: u32` field on the
  diff is removed.
- `is_atomic` → `is_atom`, `is_composed` → `is_compound` on
  `PromptElementHeader`. New bodies match on `self.kind` rather than
  checking `composed_of`.
- `DecomposeReport` shape: `element_order` → `element_kind` plus
  `body_level`; `ReportEntry.order` → `kind` (`"atom"` | `"compound"`).
- CLI human output replaces every "order N" reference with kind /
  body_level / depth as appropriate.
- All shipped fixtures under `elements/` and `Documentation/demos/`
  migrated to the v0.2 schema.

### Removed

- `order` field from `PromptElementHeader`.
- `compute_order` function and its tests.
- Error variants: `OrderRequiresField`, `HandAuthoredHigherOrder`,
  `OrderMismatch`, `AtomicityMismatch`, `CannotDecomposeAtomic`.
- The `LICENSE` (GPL-3.0) file.

### Migration

Run `oovra migrate <library-dir>` in a clean Git working tree. The
migration is in-place and recursive into compound bodies. After
running, verify with `git diff`. Any file that fails to migrate is
left untouched and reported on stderr.

### Deferred to v0.3

- **`compare` rewrite v2** with Histogram diff and duplicate-id move
  detection. v0.2's sequence-aware compare uses id-keyed maps and
  cannot disambiguate moves when the same id appears multiple times
  in a single `composed_of`.
- **Multi-renderer support** (`--render xml`,
  `--render anthropic-messages`, etc.). The `render_mode` field
  exists; only `"markdown-h2"` is supported in v0.2.
- **Content hashes** in `composed_of` for tamper-evidence.
- **Token budgeting**: header field + `compose` flag.
- **Slot system** for parameterized atoms.
- **Semantic embedding layer** for similarity-based atom retrieval.
- **`oovra-server`** as a network-accessible prompt library.
- **`oovra rename`** for safe id rewrites across a library.
- **`oovra init` + tutorial** flagged by the v0.1 ceiling analysis as
  the highest-leverage adoption win.
- **Templating** flagged by the v0.1 ceiling analysis.
- **Pre-built binaries / crates.io publication.**
- **`--legacy` flag cleanup** — stays in v0.2, removed in v0.3.
- **Dedicated `InvalidKind` error variant**: a custom-message error for
  an invalid `kind` value (e.g. `kind = "atomic"`). The existing
  `InvalidToml` path already carries serde's "unknown variant `x`,
  expected `atom` or `compound`" message, so v0.2 ships with the
  serde-default error.

### Internal

- Crate-level `#![allow(clippy::result_large_err)]` for the pre-existing
  large-error-variant lint (`toml::de::Error` inside
  `OovraError::InvalidToml`). Boxing every error path is a runtime cost
  paid for stack-size cleanliness; revisit if benchmarks justify it.

## [0.1.0] — 2026-05-09

Initial release. Four operators (Create, Compose, Decompose, Compare);
`order`-based schema; chiral `~~>>` / `~~<<` body delimiters with
strict-monotonicity escalation for nested compositions; markdown-h2
renderer; Obsidian-compatible library directory layout.
