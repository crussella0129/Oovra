---
title: "Oovra File Format Schema"
project: "[[Oovra]]"
related:
  - "[[Oovra Build Guide]]"
  - "[[Policy Node Composer]]"
  - "[[Claude Thoughts Vault]]"
tags:
  - oovra
  - schema
  - reference
  - agent-facing
  - markdown
  - toml
status: canonical
version: 0.1
audience: humans-and-agents
created: 2026-05-06
---

# Oovra File Format Schema

This is the canonical schema definition for Oovra files. It is written to be read by both humans and AI agents. If you are an agent reading this to learn how to author Oovra files, the schema rules in this document are sufficient — you do not need any external context beyond what is here.

If anything in this document conflicts with the Rust source code, **the source code is wrong** and should be fixed to match this document. This file is the contract.

---

## What an Oovra File Is

An Oovra file is a single `.md` file containing two parts:

1. A **TOML frontmatter block** at the very top of the file, delimited by `+++` on its own line at the start and `+++` on its own line at the end. This block carries the file's metadata in TOML syntax.
2. A **Markdown body** following the closing `+++` delimiter, separated from it by exactly one blank line. This block carries the file's content as freeform Markdown.

Every Oovra file has both parts. A file missing either part is not a valid Oovra file and will be rejected by the parser.

---

## The `kind` Field — The Most Important Field in the Schema

Every Oovra frontmatter block must contain a `kind` field. The value of this field tells Oovra (and any agent reading the file) what role the file plays in the system. The `kind` field discriminates which schema rules apply to the rest of the frontmatter.

Two values are valid in this version of the schema:

- `kind = "node"` — the file is a hand-authored or agent-authored atomic policy node. Its body is the prose policy itself.
- `kind = "completed"` — the file is a tool-emitted rendered prompt produced by Oovra's Compose operator. Its body is the assembled output; its frontmatter records the recipe that produced it.

A third value, `"bundle"`, is reserved for future use and must not be set by current authors.

If the `kind` field is missing, malformed, or holds any value other than `"node"` or `"completed"`, the parser will reject the file with an error.

---

## Delimiter Rules (Read This Carefully)

The frontmatter delimiter is exactly three plus signs (`+++`) on a line by themselves. **Not** three dashes (`---`). The choice of `+++` over `---` is intentional: `---` is the Jekyll/YAML-frontmatter convention, and using `+++` unambiguously signals "this is TOML frontmatter" without requiring the parser to peek inside.

Specific rules:

- The opening `+++` must be the very first line of the file. Not a blank line. Not a comment. The first line.
- The closing `+++` must be on its own line. Nothing else may appear on that line.
- After the closing `+++`, there must be exactly one blank line, then the Markdown body.
- The body itself may contain `+++` characters anywhere, including at the start of lines. The parser only cares about the first two `+++` lines in the file.

A correct minimal file looks like:

```
+++
kind = "node"
id = "example-node"
version = "1.0.0"
tags = []
+++

This is the body.
```

A common mistake to avoid: do not put any whitespace before or after the `+++` on its delimiter lines. The line must consist of exactly the three plus signs and a newline.

---

## Schema for `kind = "node"`

A node is the atomic unit of policy in Oovra. One node expresses one internally consistent rule, instruction, or fragment of guidance that can be reused across many completed policies.

### Required fields

| Field | TOML type | Constraints | Description |
|---|---|---|---|
| `kind` | string | must equal `"node"` | Discriminates this file as a policy node. |
| `id` | string | kebab-case, unique across the library | The node's stable identifier. Used in references from completed policies. Must not change once published. |
| `version` | string | valid semver (e.g. `"1.0.0"`, `"2.3.1-rc1"`) | The node's version. Bump on every meaningful body change. Used for version pinning by completed policies. |
| `tags` | array of strings | may be empty | Categorization tags. Become Obsidian tags when the library is opened as an Obsidian vault. |

### Optional fields

| Field | TOML type | Description |
|---|---|---|
| `name` | string | Human-readable display name. If absent, tools should display the `id`. |
| `description` | string | One-line summary of the node's purpose. |
| `requires` | array of strings | IDs of other nodes this one depends on. Compose may use this in v0.2 for automatic dependency expansion. |
| `conflicts_with` | array of strings | IDs of other nodes this one cannot coexist with in a single composition. |
| `author` | string | The author of the node (human name, agent name, or system identifier). |
| `created` | string | ISO 8601 date or datetime when the node was first created. |
| `notes` | string | Free-form notes from the author. Not consumed by Oovra. |

### Body rules for nodes

The body is freeform Markdown. The body holds the actual policy text — what the node "says" when included in a composition.

The body should:

- Be **internally consistent**: a node should express one coherent policy that makes sense in isolation, without requiring adjacent nodes for context.
- Be **portable**: the body should read correctly when included in any composition, not just one specific composition.
- Be **the right size**: typically one to five sentences expressing one coherent policy. Single sentences are usually too small; multi-paragraph bodies are usually too big.

The body may contain Obsidian-style wiki-links (`[[other-node-id]]`) to indicate semantic relationships to other nodes. Oovra ignores these — they're invisible to the parser — but Obsidian renders them as live links and includes them in the graph view. Use them when the relationship matters; do not force them where they don't fit.

The body may contain standard Markdown: headers, lists, code blocks, emphasis, links. There are no restrictions beyond those of Markdown itself.

### Example: a complete node file

```
+++
kind = "node"
id = "refusal-policy-strict"
version = "1.2.0"
tags = ["safety", "production", "refusal"]
name = "Strict Refusal Policy"
description = "Refuse harmful requests with a brief, non-preachy decline."
requires = ["role-declaration"]
author = "Charles Russell"
created = "2026-04-15"
+++

When asked to produce content that would cause concrete harm — including
weapons synthesis, malware, or content sexualizing minors — decline briefly
and without lecturing. Do not explain at length why the request is harmful;
a single clear sentence of decline is enough. Then offer a constructive
alternative if one exists.

This policy assumes the [[role-declaration]] node is also included in the
composition; it relies on that node to establish the assistant's general
posture toward the user.
```

---

## Schema for `kind = "completed"`

A completed policy is the rendered output of Oovra's Compose operator. It is a self-describing artifact: its frontmatter records the recipe (which nodes, in what order, at what versions), and its body holds the rendered text. Any completed policy can be Decomposed back to its recipe without consulting external state.

### Required fields

| Field | TOML type | Constraints | Description |
|---|---|---|---|
| `kind` | string | must equal `"completed"` | Discriminates this file as a completed policy. |
| `id` | string | kebab-case, unique across completed policies | The completed policy's stable identifier. |
| `generated_at` | string | ISO 8601 datetime with timezone | When this completed policy was rendered by Compose. |
| `render_mode` | string | identifies the renderer used | The renderer that produced the body. v0.1 supports `"markdown-h2"`. Future renderers will add values like `"claude-xml"`, `"plain-text"`. |
| `nodes` | array of inline tables | each entry has `id` and `version` fields | The ordered recipe. Each entry records one node that was included in this composition, in the order it was rendered. |

### Optional fields

| Field | TOML type | Description |
|---|---|---|
| `name` | string | Human-readable display name. |
| `description` | string | One-line summary of the completed policy's purpose. |
| `source_library` | string | Path to the library directory used at compose time. Useful for reproduction. |
| `composer_version` | string | Semver of the Oovra binary that produced this file. Useful for diagnosing renderer changes between versions. |
| `overrides` | array of inline tables | Records any inline body overrides applied during composition. Each entry has `node_id` (string) and `applied_override` (string). Empty or absent if no overrides were used. |
| `target_model` | string | Identifies the AI model this prompt was composed for (e.g. `"claude-opus-4-7"`, `"gpt-5"`). Informational only; Oovra does not enforce model-specific constraints. |
| `notes` | string | Free-form notes. Not consumed by Oovra. |

### The `nodes` array — most important field after `kind`

The `nodes` array is the recipe. Its order matters: it is the order in which nodes were rendered into the body. Decompose reads this array to recover the composition; Re-render uses it to regenerate the body after node changes; Compare uses it to perform structural diffs.

Each entry in the `nodes` array is a TOML inline table with two required fields:

- `id` — the ID of a node that was included.
- `version` — the version of that node that was resolved at compose time. This records what was actually used, not what was requested.

In TOML syntax, the `nodes` array is written as an array of inline tables:

```toml
nodes = [
  { id = "role-declaration", version = "1.0.0" },
  { id = "refusal-policy-strict", version = "1.2.0" },
  { id = "output-format-markdown", version = "0.3.1" }
]
```

Or, equivalently, as an array of sub-tables:

```toml
[[nodes]]
id = "role-declaration"
version = "1.0.0"

[[nodes]]
id = "refusal-policy-strict"
version = "1.2.0"

[[nodes]]
id = "output-format-markdown"
version = "0.3.1"
```

Both forms are valid and parse identically. The inline-table form is more compact for typical use; the sub-table form is more readable for compositions with many nodes or with future per-node metadata. Compose emits the inline-table form by default.

### Body rules for completed policies

The body is the rendered output produced by Oovra's renderer. It is intended to be copied directly into a system prompt field, an API call, or wherever the prompt is actually used.

For `render_mode = "markdown-h2"`, the body consists of each composed node's body wrapped in a Markdown H2 header containing the node's ID, with adjacent nodes separated by a blank line. Future render modes will produce different body structures.

The body should be treated as opaque by humans and agents editing Oovra files. **Do not hand-edit the body of a completed policy.** If the body needs to change, change the underlying node and re-run Compose. Hand-edits to a completed policy's body silently desynchronize it from its recipe; the next Compose run will overwrite them without warning.

If you find yourself wanting to hand-edit a completed policy's body, that's a signal that you should either (a) edit the underlying node and re-render, or (b) use the `overrides` field in the recipe to record an inline override that survives re-rendering.

### Example: a complete completed-policy file

```
+++
kind = "completed"
id = "coding-agent-strict"
generated_at = "2026-05-06T14:23:15Z"
render_mode = "markdown-h2"
name = "Coding Agent (Strict Refusal)"
description = "Production coding agent prompt with strict refusal policy."
source_library = "./nodes"
composer_version = "0.1.0"
target_model = "claude-opus-4-7"
nodes = [
  { id = "role-declaration", version = "1.0.0" },
  { id = "refusal-policy-strict", version = "1.2.0" },
  { id = "output-format-markdown", version = "0.3.1" },
  { id = "tone-direct", version = "1.1.0" }
]
+++

## role-declaration

You are an expert software engineer assisting a developer with code review,
debugging, and design questions. You answer concisely and ground your
answers in the specific code at hand.

## refusal-policy-strict

When asked to produce content that would cause concrete harm — including
weapons synthesis, malware, or content sexualizing minors — decline briefly
and without lecturing. Do not explain at length why the request is harmful;
a single clear sentence of decline is enough. Then offer a constructive
alternative if one exists.

## output-format-markdown

Format responses in Markdown. Use code blocks with language tags for all
code samples. Use bullet lists only when the content is genuinely a list of
parallel items.

## tone-direct

Be direct. Skip preamble. Skip apology. State conclusions before reasoning
unless the reasoning must be understood first to make sense of the
conclusion.
```

---

## Validation Rules (What the Parser Checks)

Oovra's parser performs the following checks on every file. Authors and agents should ensure their files pass these checks.

### Lexical and structural checks (apply to all kinds)

1. The file begins with `+++` on its own line.
2. A second `+++` line appears later in the file.
3. The content between the two `+++` lines is valid TOML.
4. The frontmatter contains a `kind` field whose value is a string.
5. The `kind` value is `"node"` or `"completed"`.

### Semantic checks for `kind = "node"`

1. `id` is present, is a string, and is in kebab-case (lowercase letters, digits, and hyphens only).
2. `version` is present, is a string, and parses as a valid semantic version.
3. `tags` is present and is an array of strings.
4. Every optional field, if present, has the type listed in the schema.
5. The body is non-empty after stripping whitespace.

### Semantic checks for `kind = "completed"`

1. `id` is present, is a string, and is in kebab-case.
2. `generated_at` is present, is a string, and parses as a valid ISO 8601 datetime.
3. `render_mode` is present and is a string.
4. `nodes` is present, is an array, and each entry is a table containing `id` (string) and `version` (string).
5. Every optional field, if present, has the type listed in the schema.
6. The body is non-empty after stripping whitespace.

### Library-wide checks (performed when loading a library)

1. All node `id` values across the library are unique.
2. All completed-policy `id` values across the library are unique.
3. (v0.2) All `requires` references in node frontmatter point to node IDs that exist in the library.
4. (v0.2) No `conflicts_with` cycle exists.

---

## Guidance for Agents Authoring Oovra Files

If you are an AI agent producing Oovra files, follow these rules to maximize the likelihood that your output parses correctly the first time:

**Always start with the frontmatter.** Write `+++`, then the TOML, then `+++`, then a blank line, then the body. Do not lead with the body. Do not put commentary before the opening delimiter.

**Always include the required fields for the kind you're authoring.** For nodes, that means `kind`, `id`, `version`, and `tags` — even if `tags` is an empty array (`tags = []`). For completed policies, that means `kind`, `id`, `generated_at`, `render_mode`, and `nodes`. Missing required fields are the most common cause of parse failures.

**Quote all string values.** TOML technically allows bare strings in some contexts, but always quoting strings eliminates an entire class of edge cases (such as values that look like booleans or numbers).

**Use kebab-case for all IDs.** Lowercase letters, digits, and hyphens. No spaces, no underscores, no uppercase. The parser will reject IDs that don't match this convention.

**Use semver for all version strings.** Three numeric components separated by dots, optionally followed by a pre-release suffix. Examples: `"1.0.0"`, `"2.3.1"`, `"0.1.0-alpha1"`. The string `"v1.0"` is **not** valid semver and will be rejected.

**For `generated_at`, use ISO 8601 with timezone.** Example: `"2026-05-06T14:23:15Z"`. The `Z` suffix indicates UTC. Local-time strings without a timezone are rejected.

**If you are authoring a node, write a body that is internally consistent and portable.** A node body should make sense when included in any composition. Avoid phrases like "as I said above" or "the next section" — those break when the node is composed in a different position.

**If you are producing a completed policy, only do so via Oovra's Compose operator.** Hand-authored completed policies are technically valid if they pass schema validation, but they bypass the recipe-tracking machinery and break the Decompose round-trip. If you need to assemble a prompt programmatically, call `oovra compose`; do not synthesize the file directly.

**When in doubt, run `oovra compose --text <node-id>` on a node you just authored.** This is the staged-form inspection pattern: Compose loads the node, validates it, and renders just its body to stdout — without writing anything to disk. If the node parses, you'll see the body. If it doesn't, you'll see a clear error message identifying what's wrong. This is Oovra's built-in validation path; there is no separate `oovra inspect` command because there doesn't need to be one. Use this as your validator before declaring a task complete.

---

## Versioning of This Schema

This document describes Oovra schema version `0.1`. Future schema versions will preserve backward compatibility where possible:

- New optional fields may be added at any time.
- New `kind` values may be added (the planned `"bundle"` kind is the next one).
- New required fields will only be added in major schema-version bumps and will be accompanied by migration guidance.
- Existing field types will not change.
- Existing field semantics will not change.

The schema version is **not** recorded in individual Oovra files in v0.1 because there is only one version. When v0.2 ships, a `schema_version` field will be added to the optional fields list, and files without it will be assumed to follow v0.1 rules.

---

## Quick Reference Card

```
+++
kind = "node" | "completed"
id = "kebab-case-id"
version = "semver"               # nodes only
generated_at = "ISO 8601"        # completed only
render_mode = "markdown-h2"      # completed only
nodes = [ {id=..., version=...} ] # completed only
tags = []                        # nodes only, may be empty
# ... optional fields ...
+++

(body — freeform Markdown for nodes;
 renderer output for completed policies)
```

If you remember nothing else, remember:

- `+++` not `---`
- `kind` decides everything
- IDs are kebab-case
- Versions are semver
- Run `oovra compose --text <node-id>` to verify
