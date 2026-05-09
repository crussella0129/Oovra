+++
name = "Oovra File Format Schema"
order = 0
id = "oovra-schema"
version = "0.1.0"
meta = "Canonical schema for Oovra prompt-element files. Read this first if you are an agent authoring Oovra files."
+++

# Oovra File Format Schema (v0.1)

This is the canonical schema for Oovra files. It is written for both humans and AI agents. If you are an agent authoring Oovra files, the rules in this document are sufficient — you do not need any external context.

If anything in this document conflicts with the Rust source, **the source is wrong** and should be fixed to match this document. This file is the contract.

---

## What an Oovra File Is

An Oovra file is a single `.md` file with two parts:

1. A **TOML frontmatter block** delimited by `+++` on its own line at the top and `+++` on its own line at the end.
2. A **Markdown body** following the closing `+++`, separated by exactly one blank line.

A file missing either part is rejected.

---

## One Schema, One Discriminator: `order`

Every Oovra file is a **prompt element**. There is no `kind` field. Instead, a numeric `order` field tells the parser (and the agent) what role the file plays:

- **`order = 0`** — atomic, hand-authored. A self-consistent sentence or paragraph that holds together on its own.
- **`order = 1`** — a composition of order-0 elements, produced by `oovra compose`.
- **`order = N` (N ≥ 2)** — a composition produced by `oovra compose` from inputs that include at least two elements at order `N-1`.

You cannot hand-author a file with `order >= 1`: the parser rejects any file with `order > 0` that lacks a `composed_of` recipe. Use `oovra compose` to produce higher-order elements.

### How `order` is computed

When `oovra compose` produces an element from inputs, it computes the output's order:

```
let H = max(input.order for input in inputs)
let C = count of inputs whose order == H
output_order = if C > 1 then H + 1 else H
```

Worked examples:

| Inputs | Result | Why |
|---|---|---|
| 3× order-0 | order 1 | 3 peers at the top level → climb |
| 2× order-1 | order 2 | 2 peers at top → climb |
| 1× order-1 + 4× order-0 | order 1 | only 1 peer at top → no climb |
| 1× order-2 + 1× order-1 + 5× order-0 | order 2 | only 1 peer at top → no climb |
| 2× order-2 + 1× order-1 | order 3 | 2 peers at top → climb |

The order encodes "compositional depth" — how many distinct levels of peer composition went into producing this element.

---

## Delimiter Rules

The frontmatter delimiter is exactly `+++` on a line by itself — **not** `---`. The choice is intentional: `---` signals YAML; `+++` signals TOML; an Oovra file is unambiguous from its first line.

Strict rules:
- The opening `+++` must be the first line of the file.
- The closing `+++` must be on its own line.
- Exactly one blank line follows the closing `+++`, then the body.
- The body may contain `+++` characters anywhere (they're just text).

---

## Schema for All Prompt Elements

### Required fields

| Field | TOML type | Constraints |
|---|---|---|
| `name` | string | non-empty; the human-readable display name |
| `order` | non-negative integer | 0 for atomic; tool-computed for composed |
| `id` | string | kebab-case (lowercase, digits, hyphens; no leading/trailing/double hyphens); unique across the library |
| `version` | string | valid semver (e.g. `"1.0.0"`, `"2.3.1-rc1"`) |
| `meta` | string | the customizable description; may be empty (`""`) |

### Required when the element is composed (i.e. has a `composed_of` recipe)

| Field | TOML type | Constraints |
|---|---|---|
| `generated_at` | string | RFC 3339 timestamp (e.g. `"2026-05-09T14:23:15Z"`) |
| `render_mode` | string | identifies the renderer; v0.1 supports `"markdown-h2"` |
| `body_level` | non-negative integer | the **physical body delimiter level** — always `max(input.order) + 1` at compose time. Distinct from `order`; see "Two integers" below. |
| `composed_of` | array of inline tables `{id, version}` | the immediate inputs (one level down) in compose order; non-empty |

These four fields are **jointly required**: when `composed_of` is present, the other three must also be present. When `composed_of` is absent, all four must be absent (and `order` must be `0` — see invariants below).

### Invariants between fields

- A hand-authored element has `order = 0`, no `composed_of`, no companions. The validator rejects any other shape.
- A composed element has `composed_of` plus all three companions. Its `order` is computed by the formula above; its `body_level` is computed independently (see below).
- Therefore: `composed_of.is_none()` ⇒ `order == 0`, and the converse direction is permitted (a composed element CAN have `order = 0` — the single-input edge case from the formula).

### Two integers, not one: `order` vs `body_level`

These look similar but answer different questions:

- **`order`** is the *logical* compositional depth. It only climbs when the user's formula's "≥2 peers at the maximum input order" condition is met. This is the user-meaningful number — "how many distinct levels of composition went into this artifact."
- **`body_level`** is the *physical* delimiter level used by the body. It is always `max(input.order) + 1`. This is what makes the body parser unambiguous.

For homogeneous compositions (all inputs at the same order), the two coincide. For mixed-order compositions where the maximum-order count is 1, they diverge: `order` stays at the input maximum, `body_level` climbs anyway. The divergence is what keeps the body parser from colliding with inner delimiters.

### Body rules

- **For `order = 0`**: the body is freeform Markdown — the actual prompt text. It must be non-empty after stripping whitespace.
- **For composed elements** (those with a `composed_of` field): the body is a sequence of K complete sub-element files (each with their own frontmatter), wrapped in chiral level-aware delimiters whose tilde count is set by `body_level`. See below.

---

## Body Delimiter Scheme (composed elements only)

When `oovra compose` produces a composed element, its body is the **concatenation of the full file content of each input** (frontmatter + body of each input), wrapped in chiral delimiters whose tilde count is determined by the parent's `body_level` field.

| `body_level` | Open | Close |
|---|---|---|
| 1 | `~~>>` | `~~<<` |
| 2 | `~~~>>` | `~~~<<` |
| N | `(N+1)` tildes + `>>` | `(N+1)` tildes + `<<` |

The parser reads `body_level` from the element's header and scans for delimiters with exactly that tilde count.

**Properties:**

- **Chiral**: open ≠ close. Mismatched delimiters are a parse error.
- **Strictly monotonic**: a level-N delimiter has *more* tildes than any level less than N. This means the parser for an order-N body, scanning for `(N+1)` tildes, ignores every inner level-(N−k) delimiter.
- **Each chunk is itself a complete Oovra file.** This means `decompose --full` can recursively split bodies and recover every leaf — including all metadata (name, version, meta) — without consulting any external state.

### Example: an order-1 body

```
~~>>
+++
name = "Strict Refusal Policy"
order = 0
id = "refusal-policy-strict"
version = "1.0.0"
meta = "..."
+++

(the order-0 body text)
~~<<
~~>>
+++
name = "Tone: Direct"
order = 0
id = "tone-direct"
version = "1.0.0"
meta = "..."
+++

(the order-0 body text)
~~<<
```

---

## Validation Rules (What the Parser Checks)

### Lexical and structural (all elements)

1. The file begins with `+++` on its own line.
2. A second `+++` line appears later.
3. The content between is valid TOML.
4. `name`, `order`, `id`, `version`, `meta` are present with correct types.
5. The body is non-empty after stripping whitespace.

### Semantic

1. `id` matches the kebab-case grammar.
2. `version` parses as semver.
3. `name` is non-empty.
4. If `composed_of` is present: `generated_at` (RFC 3339), `render_mode`, and `body_level` are also present; `composed_of` is non-empty; every entry has a kebab-case `id` and a semver `version`.
5. If `composed_of` is absent: `generated_at`, `render_mode`, and `body_level` must all be absent, AND `order` must be `0`. This rejects hand-authored claims of higher orders without a recipe.

### Library-wide

1. All `id` values across the library are unique.

---

## Operators (the Four Verbs)

The CLI is `oovra`, with four subcommands:

- **`oovra create`** — author a new order-0 element (`--new`) or label an existing `.md` file (`--label`). Always produces order 0.
- **`oovra compose`** — combine ordered inputs into a higher-order element. Computes the output order. Three modes: writes a file (default), prints body text (`--text`), or regenerates the body of an existing composed file (`--re-render <path>`).
- **`oovra decompose`** — split a composed element. Default returns the immediate inputs (one level down). `--full` recursively writes a folder tree all the way to order-0 leaves.
- **`oovra compare`** — diff two prompt elements. Same-order-0 → content diff. Same-order-≥1 → structural diff over `composed_of` (added / removed / version-changed inputs). Different orders → refused.

Validation is internal to every operator. There is no separate `validate` or `inspect` command — running `oovra compose --text <id>` on a single element loads it, validates it, and prints the body without writing anything. That is the staged-form inspection path.

---

## Guidance for Agents

If you are an LLM authoring Oovra files, follow these rules to maximize first-shot correctness:

- **Always include all five required fields** (`name`, `order`, `id`, `version`, `meta`). `meta = ""` is fine; missing `meta` is a parse error.
- **Hand-authored files are always `order = 0`.** Do not write higher-order files by hand. Use `oovra compose`.
- **Quote all string values.** TOML allows bare strings in some contexts but always quoting eliminates a class of edge cases.
- **Use kebab-case for all IDs.** Lowercase letters, digits, hyphens; no leading/trailing/double hyphens.
- **Use semver for all versions.** `"1.0.0"`, not `"v1.0"` or `"1.0"`.
- **Write portable bodies.** Order-0 bodies should make sense in any composition. Avoid "as I said above" / "the next section" — those break when the element is composed in a different position.
- **When in doubt, run `oovra compose --text <id>`** on the element you just authored. If it parses, you'll see the body. If not, you'll see a clear error.

---

## Quick Reference Card

```toml
+++
name = "Human-Readable Name"
order = 0
id = "kebab-case-id"
version = "1.0.0"
meta = "optional description; may be empty string"
# When the element has a recipe, the following are also required:
# generated_at = "RFC 3339"
# render_mode = "markdown-h2"
# body_level = 1                # = max(input.order) + 1
# composed_of = [{ id = "...", version = "..." }, ...]
+++

(body — freeform Markdown for atomic order-0 elements;
 wrapped sub-element files for composed elements)
```

If you remember nothing else:

- `+++` not `---`
- `order` decides everything; `order = 0` for hand-authored
- IDs are kebab-case
- Versions are semver
- Run `oovra compose --text <id>` to verify
