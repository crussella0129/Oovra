---
title: "Oovra Build Guide"
project: "[[Oovra]]"
related:
  - "[[Carbide]]"
  - "[[Cellpiler]]"
  - "[[Claude Thoughts Vault]]"
  - "[[Policy Node Composer]]"
  - "[[String Operators sheet]]"
  - "[[Obsidian Vault]]"
tags:
  - rust
  - prompt-engineering
  - composer
  - architecture
  - build-guide
  - oovra
  - markdown
  - obsidian-compatible
status: spec
version: 0.3
author: Charles Russell
audience: self
created: 2026-05-06
supersedes: "Oovra Build Guide v0.2 (had Inspect as a fifth operator)"
---

# Oovra Build Guide

> *"To be made in the image of the Creator is to be a creator of very good things yourself."*
>
> An **œuvre** is a body of work — the collected output of a maker. Oovra (the phonetic spelling, used in code and CLI) treats system prompts as composed works: assembled from named, versioned policy nodes that form your personal corpus. Each prompt you ship is an entry in your œuvre.

A step-by-step prose guide for building Oovra, a system-prompt composer in Rust. Oovra operates on a single file format — Markdown with TOML frontmatter — and treats every artifact in the system as the same kind of file. Humans edit them, agents author them, the Rust tool reads and writes them.

The build guide is deliberately written in prose. Every step describes *what to do* and *why* — the syntax is left for the compiler and the documentation to teach. This is a learn-by-doing artifact for the [[Architect/Builder Gap]] problem.

---

## Architectural Decisions Locked In

Before you write any code, internalize these. They're load-bearing for everything downstream.

**One file format only.** Every Oovra artifact is a `.md` file with a TOML frontmatter block delimited by `+++`. There is no JSON. There is no YAML. There is no second format. This is the most important decision in the system; preserve it ruthlessly.

**The header carries the meaning, the body carries the content.** A file's `kind` field in the frontmatter tells Oovra what role the file plays. Three kinds exist: `node` (a hand-authored or agent-authored atomic policy), `completed` (a tool-rendered prompt assembled from nodes), and `bundle` (a future feature — a collection of related nodes, deferred to v0.2). The body is freeform Markdown.

**Three audiences, one format.** Humans edit Oovra files. Agents author Oovra files. The Rust tool consumes Oovra files. All three touch the same physical files. The format must be forgiving enough for humans, structured enough for parsers, and statistically natural enough for agents to author correctly. Markdown-with-TOML-frontmatter is one of the few formats that genuinely serves all three.

**Obsidian-compatible by construction.** Because every Oovra file is a valid Markdown file, the entire library is also a valid Obsidian vault. Wiki-style links (`[[node-id]]`) in node bodies become live navigation in Obsidian. Tags in frontmatter become Obsidian tags. The graph view shows your prompt architecture. This is a free property of the format choice; do not break it.

**Four operators, one binary.** The CLI is `oovra`, with four subcommands: Create, Compose, Decompose, Compare. Each maps to a specific operation on Oovra files. The full operator surface fits on one help screen.

**Validation is internal, never a CLI verb.** Every operator validates its inputs and outputs as part of its normal work, and emits a clear error if anything fails. There is no separate "inspect" or "validate" subcommand because there doesn't need to be one — agents and humans can both read the file directly (it's plain Markdown) or run `oovra compose --text <node-id>` to render a node and confirm it parses. The Unix-philosophy version: text is the universal interface; do not build a parallel structured-data API alongside the prose interface when both your humans and your agents can read prose.

---

## Part 1 — Documentation Resource List

Before writing any code, bookmark these. Read the first one or two pages of each *now* so you know where to look later. You don't need to read them deeply yet — you need to know they exist and roughly what they cover.

### Rust language and ecosystem

- **The Rust Book** — `https://doc.rust-lang.org/book/` — the canonical introductory text. Chapters 1–10 cover everything you need for this project. Chapters on ownership, structs, enums, error handling, and modules are the load-bearing ones.
- **Rust by Example** — `https://doc.rust-lang.org/rust-by-example/` — runnable examples for nearly every concept. Better than the Book for "show me what this looks like in practice."
- **The Rust Standard Library reference** — `https://doc.rust-lang.org/std/` — the documentation for everything that comes built in. You'll spend most of your time in `std::fs`, `std::path`, `std::collections`, and `std::io`.
- **The Cargo Book** — `https://doc.rust-lang.org/cargo/` — Cargo is Rust's build tool and package manager.
- **Rust API Guidelines** — `https://rust-lang.github.io/api-guidelines/` — naming conventions, error handling patterns, idiomatic API design. Skim it; don't read it cover to cover.

### Crates you will use

For each crate below, the canonical documentation lives at `https://docs.rs/<crate-name>`. That URL pattern works for every published Rust crate; learn it.

- **`serde`** — `docs.rs/serde` — the universal serialization framework. The derive macros (`#[derive(Serialize, Deserialize)]`) are what make TOML parsing trivial.
- **`toml`** — `docs.rs/toml` — TOML parser, used for the frontmatter block of every Oovra file. This is the only structured-data format parser you need.
- **`gray_matter`** — `docs.rs/gray_matter` — Markdown frontmatter parser. Handles TOML, YAML, and JSON frontmatter automatically. You'll use it as the first-pass splitter that separates frontmatter from body, then hand the frontmatter to the `toml` crate.
- **`walkdir`** — `docs.rs/walkdir` — recursive directory traversal. Simpler than `std::fs::read_dir` for "give me every file under this path."
- **`clap`** — `docs.rs/clap` — command-line argument parsing. Use the derive feature (`features = ["derive"]`) — it lets you define your CLI as a struct.
- **`anyhow`** — `docs.rs/anyhow` — ergonomic error handling for application code. Use this in your binary; consider `thiserror` later if you split into a library crate.
- **`thiserror`** — `docs.rs/thiserror` — for defining structured error enums. Pair with `anyhow` if your project grows.
- **`semver`** — `docs.rs/semver` — semantic version parsing and comparison. You'll need this for version-pin matching in Compose.
- **`similar`** — `docs.rs/similar` — text diffing library, for the human-readable diff output of Compare. Add when you reach Stage 3.
- **`owo-colors`** — `docs.rs/owo-colors` — terminal color output for the Compare CLI. Optional; cosmetic.
- **`indexmap`** — `docs.rs/indexmap` — a hash map that preserves insertion order. Optional; useful if you need ordered iteration over the library.

### Markdown, frontmatter, and TOML

- **CommonMark spec** — `https://spec.commonmark.org/` — the standardized Markdown spec. You don't need to memorize it, but know it exists when you have a "how does Markdown actually parse X" question.
- **Frontmatter conventions** — there is no formal spec for Markdown frontmatter. The de facto standards are: a block delimited by `---` containing YAML (Jekyll convention), or a block delimited by `+++` containing TOML (Hugo convention). Oovra uses `+++` and TOML for unambiguous parsing.
- **TOML specification** — `https://toml.io/en/v1.0.0` — the official TOML spec. Short, readable, browseable in 20 minutes. The "Keys", "String", "Array", and "Table" sections are the ones you'll use.
- **Obsidian help — Internal links** — `https://help.obsidian.md/Linking+notes+and+files/Internal+links` — the wikilink syntax (`[[note-name]]`) you should use in node bodies. Oovra never parses wikilinks itself — they're just text to Oovra — but Obsidian will render them as live links if your library is opened as a vault.

### Reference for prompt structure (your sheet)

- **String Operators sheet** — `https://docs.google.com/spreadsheets/d/1fOlZ3YK-Fk7w4OU3tagHfohqSGkAPGL3CrboVoFcnZA` — the algebra you're porting. Keep this open in a tab while building Stage 3.

---

## Part 2 — Build Instructions

The build is in four stages. Stage 1 defines the file format (the Oovra Header schema) and writes the parser that handles all kinds — this is internal infrastructure that every later operator depends on. Stage 2 builds Create — the simplest operator, which lets you author new files end-to-end. Stage 3 builds Compose and Decompose — the JOIN and SPLIT operations from your sheet. Stage 4 builds Compare — the FORWARD-DIFF operation, with kind-aware dispatch.

Validation is not a separate stage and not a separate operator. The parser built in Stage 1 is called by every operator that follows. When something fails to parse, the operator that called the parser surfaces the error to the user. The "validate this file before doing anything with it" workflow is achieved by running `oovra compose --text <node-id>` (which loads, validates, and renders the node, but does not write anything) and reading the output or error. This is the staged-form inspection pattern: agents and humans confirm a file is well-formed by asking Oovra to do the smallest possible operation on it and observing the result.

You do not need to finish a stage before sketching the next one on paper. You *do* need to finish a stage before writing code for the next one. Write tests as you go; do not save them for the end.

### Stage 1 — Define the Oovra Header format and write a parser

#### Step 1.1 — Specify the file format on paper before touching Rust

Open your notebook. Write down — by hand — the exact shape of the Oovra Header for each of the two kinds you're shipping in v0.1.

For a **node**, the header must contain: a unique identifier (string, kebab-case, used as the node's name throughout the system), a semantic version string (e.g. `"1.0.0"`), a kind discriminator set to the literal string `"node"`, and a list of tags. The header may optionally contain: a human-readable name, a one-line description, a list of dependency IDs (other nodes this one assumes will be present), and a list of conflict IDs (other nodes this one cannot coexist with). The body is the prose policy itself — freeform Markdown, with `[[wiki-links]]` allowed for Obsidian compatibility.

For a **completed policy**, the header must contain: a unique identifier (string, kebab-case, naming this completed policy), a kind discriminator set to the literal string `"completed"`, a generation timestamp (ISO 8601), a render mode (a string identifying which renderer was used, e.g. `"markdown-h2"`), and an ordered array of node references. Each node reference is a TOML inline table or a sub-table containing the referenced node's ID and the version that was resolved at compose time. The header may optionally contain: a human-readable name, a description, the source library path that was used, and a list of any inline overrides applied during composition. The body is the rendered prompt as Markdown.

Write each field's name, its TOML type, and whether it's required or optional in your notebook. This becomes your schema. Do not add fields you cannot justify; over-specified schemas are how projects die in v0.1.

#### Step 1.2 — Sketch one example file of each kind on paper

Before any code: hand-write one example node file and one example completed-policy file in your notebook. Use realistic content — a refusal policy node, a completed coding-agent prompt — not placeholders. Look at them. Are they readable? Could you imagine an agent producing them correctly with only a schema description to go on? Could you imagine yourself editing one without a manual?

If anything looks awkward, fix the schema in Step 1.1 before continuing. This is the cheapest moment to revise the format. After Stage 2 ships, format changes get expensive.

#### Step 1.3 — Decide your delimiter convention and stick to it

Oovra uses `+++` (three plus signs) on a line by themselves to delimit the TOML frontmatter from the body. The opening delimiter must be the very first line of the file. The closing delimiter must be on its own line, followed by a blank line, followed by the body.

Why `+++` and not `---`: `---` is the Jekyll/YAML convention. Using `+++` unambiguously signals "this is TOML frontmatter" without requiring the parser to peek inside. It also means you can never accidentally confuse Oovra files with YAML-frontmatter files in other tools. Obsidian supports both delimiters natively, so you lose nothing on the Obsidian-compatibility side.

Write this convention down. Every Oovra file uses `+++`. No exceptions.

#### Step 1.4 — Scaffold the Rust project

Run `cargo new oovra` to create a binary project. Add the dependencies from Part 1 to your `Cargo.toml`: serde with the derive feature, toml, gray_matter, walkdir, clap with the derive feature, anyhow, and semver. You'll add `similar` and `owo-colors` later when you reach Stage 4. Do not add crates speculatively.

Inside the `src` directory, plan your module layout on paper before creating files. A reasonable layout: a `header` module for the frontmatter type and its parser, a `node` module for the Node-specific logic, a `completed` module for the Completed-specific logic, a `library` module for loading nodes from disk, a `render` module for the Compose operator, a `diff` module for the Compare operator, and `main.rs` for the CLI entry point. Each module gets its own file. This is the conventional Rust layout.

#### Step 1.5 — Define the header data structures

In the `header` module, define the Rust types that mirror your schema from Step 1.1.

Start with an enum called `Kind` that has two variants: `Node` and `Completed`. Annotate it with serde's rename rules so the variants serialize as lowercase strings (`"node"` and `"completed"`). This enum is the single source of truth for "what kind is this file"; every operator that needs to dispatch on kind reads this field.

Define a struct called `NodeHeader` whose fields mirror the required and optional fields you wrote down for nodes. Define a struct called `CompletedHeader` whose fields mirror the required and optional fields for completed policies. Each of these gets `#[derive(Serialize, Deserialize, Debug, Clone)]`.

The cleanest expression of "a header is one of two shapes" is a Rust enum that wraps both: define a top-level `Header` enum with variants `Node(NodeHeader)` and `Completed(CompletedHeader)`. Use serde's tagged enum representation with `#[serde(tag = "kind", rename_all = "lowercase")]` so that the `kind` field in the TOML automatically discriminates which variant to deserialize into. This is the move that makes the rest of the system tractable: once the header is parsed, the type system enforces that you can never accidentally treat a node as a completed policy or vice versa.

Define a top-level `OovraFile` struct with two fields: a `header: Header` and a `body: String`. This is the in-memory representation of any Oovra file. Every operator takes and produces values of this type.

#### Step 1.6 — Write the parser function

Write a function that takes a file path, reads the file from disk, splits the frontmatter block from the body using gray_matter (which knows how to handle `+++` delimiters), parses the frontmatter as TOML into your `Header` enum, captures the body as a separate string, and returns a fully populated `OovraFile` or a structured error.

The function signature in prose: takes a reference to a path, returns a result whose ok variant is `OovraFile` and whose error variant carries enough information to point at the specific file and the specific failure (no frontmatter found, malformed TOML, missing required field, unrecognized `kind` value, missing version on a node, missing node-references array on a completed policy). Use `anyhow::Context` to attach the file path to every error; debugging is impossible without it.

Pay special attention to error messages. Because agents will use Oovra, error messages are *part of the agent-facing API*. An error like "Field 'version' is missing in node loaded from `nodes/refusal.md`" is actionable for an agent. An error like "TOML deserialization failed" is not. Spend time here.

#### Step 1.7 — Write a serializer function

The inverse of Step 1.6: takes an `OovraFile`, serializes the header to TOML, wraps it in `+++` delimiters, appends the body, and returns the resulting string. Pair it with a function that writes the string to a file path. Keep these two functions separate — the string-producer is pure and trivially testable, the file-writer is the I/O wrapper.

Test round-tripping: parse a fixture file, serialize it back out, parse the serialized output, assert equality. This catches subtle serde mistakes (field ordering, missing serializers on optional fields, unexpected key renaming) immediately.

#### Step 1.8 — Write the library loader

In the `library` module, write a function that takes a directory path, walks it recursively using walkdir, calls your parser on every file with a `.md` extension, separates the results into nodes and completed policies based on the `kind` field, and returns a struct containing both collections.

For v0.1, fail loudly: one bad file aborts the load. You will appreciate this discipline later. Validate that node IDs are unique across the library; duplicate IDs are an error, not a warning. A library with two nodes claiming the same ID has no defined behavior, and you should refuse to load it rather than silently picking one.

Return loaded nodes in a `HashMap` keyed by ID. The order of nodes in the library doesn't matter for v0.1; only the order specified by a composition does.

#### Step 1.9 — Write five real policy nodes by hand

Stop coding. Open your favorite Markdown editor — or Obsidian itself — and write five policy node files following the format you specified. Real ones, not placeholder ones. Suggested set: a role declaration, a refusal policy, an output formatting rule, a tone instruction, and an example block.

Use `[[wiki-links]]` in the bodies where it makes semantic sense (e.g. a refusal policy node body might reference `[[role-declaration]]` to indicate the relationship). Oovra ignores wiki-links — they're just text to the parser — but Obsidian will render them as live links, and your future self will thank you when navigating the library.

Save them to a `nodes/` directory at the project root. These become your test corpus and your dogfood material. You will discover problems with your schema only by authoring real content against it. If your schema feels wrong while authoring, go back to Step 1.1 and revise.

### Stage 2 — Build Create

This is the simplest operator. Building it first proves the parse-validate-serialize loop end-to-end (using the parser from Stage 1) before you tackle the more interesting JOIN and DIFF operations. There is no separate validation operator — when you want to confirm a file is valid, you'll use `oovra compose --text` (built in Stage 3) to render it and read the output or error. For Stage 2, you confirm validity by re-parsing your own output through the Stage 1 parser as part of Create's normal post-write check.

#### Step 2.1 — Implement Create

Create has two modes, matching what you sketched in your architecture diagram.

The first mode, `oovra create --pnode <id>`, scaffolds a new policy node from scratch. It prompts for (or accepts via flags) the required header fields — version, tags, optional name and description — generates a starter file with a TODO body, and writes it to the library directory. The body is just a placeholder line like `<!-- TODO: write the policy here -->` so the file parses correctly even before the human or agent fills it in.

The second mode, `oovra create --label <path>`, takes an existing Markdown file (one without an Oovra header), prompts for the required fields, prepends a generated header, and writes the file back. This is the "bring my existing prompts into Oovra" path. It should refuse to overwrite if the file already has an Oovra header, unless `--force` is passed.

Both modes produce a valid Oovra node file as their output. After writing, Create reads the file back through the Stage 1 parser and confirms it parses cleanly. If it doesn't, Create deletes the file (or restores the original in `--label` mode) and reports the error. This post-write check is internal — it doesn't surface as a separate command, it's just how Create proves its own work.

#### Step 2.2 — Wire up the CLI for Stage 2

In `main.rs`, define a clap-derive struct with one subcommand so far: `create`. Each subcommand maps to one function in your library. The subcommand's main job is: parse arguments, call one library function, print the result, exit. If the subcommand's body grows beyond fifteen lines, push logic down into the library.

Test the end-to-end flow: use `oovra create --pnode test-node` to scaffold a file, edit the body in your editor, then re-parse it manually with a small test that calls your Stage 1 parser directly. (You don't have `compose --text` to use as a one-shot validator yet — that's Stage 3. For now, "did Create produce a parseable file" is the test, and you run it with a unit test or by writing a tiny throwaway script that calls the parser.) This is your first proof that the system works end-to-end.

### Stage 3 — Build Compose and Decompose

These are the JOIN and SPLIT operators from your sheet, implemented at the Oovra-file level.

#### Step 3.1 — Specify Compose's input and output shapes

Compose takes two inputs: a *recipe* (which can be either an existing completed-policy file used as a template, or an inline list of node IDs passed via CLI flags) and a library reference. It produces one output: a completed-policy file.

The clean v0.1 ergonomic: `oovra compose --output <path> <node-id-1> <node-id-2> ... <node-id-n>` composes the listed nodes in order and writes a completed policy to the given path. The completed policy's header records the IDs and resolved versions of the nodes that went into it.

A second mode, `oovra compose --re-render <path-to-completed>`, reads an existing completed-policy file's header, re-resolves all the referenced nodes against the current library state, and overwrites the body with the freshly rendered output. This is "regenerate after node changes." If any referenced node is missing from the library or its version no longer matches the pin, Compose fails with a clear error rather than silently substituting.

A third mode is the one your last message added: `oovra compose --text <node-id-1> <node-id-2> ... <node-id-n>` (or `--text` combined with `--re-render`) outputs only the rendered body to stdout, with no header. This is the "give me the prompt I can paste into Claude" path. The text mode is one-way — the output is not a valid Oovra file, by design.

#### Step 3.2 — Write the resolver

The resolver is the function that takes a list of node IDs (with optional version pins) and a library, and produces a list of resolved node bodies in order. For each node ID, look it up in the library, check that the node's version satisfies the pin (if any), and produce the body in the original order.

Version pinning semantics for v0.1: a pin is an exact string match on the node's version field. Do not implement semver range matching yet (`^1.2.0`, `>=1.0.0, <2.0.0`); add that later when you actually feel the need. The `semver` crate is in your dependencies for when you do.

The resolver returns either a list of resolved nodes or a structured error. Possible errors: a referenced ID is not in the library, a version pin doesn't match the library's version of that node, a circular dependency exists (if you implement dependency expansion in v0.1, which you probably shouldn't). Each error must carry the offending node ID and a clear message.

#### Step 3.3 — Write the renderer

The renderer takes the resolved list of nodes and produces the final body as a Markdown string. For v0.1, support exactly one render mode: each node's body is wrapped with a Markdown H2 header containing the node's ID, separated from adjacent nodes by a blank line. Boring on purpose. Fancy rendering is a v0.2 problem.

The renderer is pure: it takes data, returns a string, no I/O. This makes it trivial to test.

#### Step 3.4 — Wire Compose to produce the full Oovra file

The full Compose flow: take the input node IDs, run them through the resolver, run the resolved nodes through the renderer to produce the body string, construct a `CompletedHeader` with the metadata (generation timestamp via `chrono` or `std::time`, render mode = `"markdown-h2"`, the resolved node references with their actual versions), wrap header and body into an `OovraFile`, serialize, write to disk.

The `--text` flag short-circuits at the renderer step: skip the header construction, skip the file write, just print the rendered body to stdout.

#### Step 3.5 — Implement Decompose

Decompose is the inverse of Compose, and it's almost trivial because of how the header is structured. It takes a path to a completed-policy file, parses the header (which already contains the ordered list of node references with versions), and outputs the recipe in a useful form.

For v0.1, Decompose's output is a structured report (printable to stdout in human-readable form, or JSON via `--format=json`) listing the node IDs and versions that compose the policy. Optionally, with `--extract`, it can also write each referenced node's current body to disk under a directory of your choosing — useful for "show me what these nodes currently look like."

The deeper move that Decompose enables: because every completed policy is self-describing via its header, you never need to keep the original recipe around. The completed policy *is* the recipe, plus the rendered output. Decompose lets you go backward whenever you need to.

#### Step 3.6 — Render a real prompt and read it

Use the tool to compose a real completed policy from your five hand-written nodes. Read the output. Does it look like a usable system prompt? If yes, you're done with Stage 3. If no, identify exactly what's wrong — formatting, ordering, missing structure — and fix only that. Do not slip into building features you imagined wanting.

Save the rendered completed policy to disk. You'll need it for Stage 4.

### Stage 4 — Build Compare

Compare implements the FORWARD-DIFF operator from your sheet. The key design choice is *kind-aware dispatch*: the diff strategy depends on what kinds of files are being compared.

#### Step 4.1 — Map the four sheet operators onto Oovra files

Open your sheet. For each operator, write down — in your notebook — the equivalent operation on Oovra files:

- **JOIN** (cells A1–G1, producing the delimited string in H1) maps to **Compose** — combine ordered nodes into a completed policy. *Already implemented in Stage 3.*
- **SPLIT** (cell B3 producing C3–H3) maps to **Decompose** — recover the node references from a completed policy. *Already implemented in Stage 3.*
- **UNIQUE-across-array** (cells B5–C8 producing D5) maps to a *library audit* operation — find all unique node IDs referenced across multiple completed policies. Defer to v0.2 unless you find an immediate use.
- **FORWARD-DIFF** (cells B10 and C10 producing D10) maps to **Compare** — given two Oovra files, report what's in the second that wasn't in the first. *This is Stage 4's main work.*

#### Step 4.2 — Decide the comparison modes

Compare must dispatch based on the kinds of the two input files. Three meaningful comparisons exist:

**Node vs. node**: a content diff. Use the `similar` crate's `TextDiff` API to produce a unified-diff-style output of the body text. Also report any frontmatter field differences (different version, different tags, different description). Output is a colored unified diff plus a structured field-change report.

**Completed vs. completed**: a structural diff. Compare the ordered list of node references in each header. Report three categories: *added* (in second, not in first by ID), *removed* (in first, not in second by ID), and *modified* (same ID, different version pin). This is the diff that surfaces "two completed policies that look totally different are actually the same composition at different node versions" — the case you specifically wanted to be able to detect.

**Node vs. completed**: ambiguous, almost always a mistake. The right interpretation is "is this node referenced by this completed policy, and if so, what version?" — which is more of a query than a diff. For v0.1, refuse this comparison with a clear error message that suggests the user probably meant to compare two files of the same kind. If you find yourself wanting it later, add it as a v0.2 feature.

The dispatch logic is simple: parse both files, read their `kind` fields, route to the appropriate diff function. The kinds are exactly the field you put in the header in Stage 1, doing exactly the work it was designed to do.

#### Step 4.3 — Implement the diff functions

In the `diff` module, write three functions:

- `diff_nodes(a: &NodeHeader, a_body: &str, b: &NodeHeader, b_body: &str) -> NodeDiff` — body diff plus header field diff.
- `diff_completed(a: &CompletedHeader, b: &CompletedHeader) -> CompletedDiff` — structural diff over node references.
- A top-level dispatch function that takes two `OovraFile` values and returns a `DiffReport` enum (variants for each diff type, plus an error variant for incompatible kinds).

Each diff function is pure: it takes parsed data, returns a structured diff result, no I/O. The CLI is a thin wrapper that reads the files, calls the dispatcher, and prints the result.

#### Step 4.4 — Wire Compare to the CLI

Add a `compare` subcommand to clap. It takes two file paths as positional arguments. It reads both files, parses their frontmatter to determine kinds, dispatches to the appropriate diff function, and prints the result. Add a `--format=json` flag for agents and downstream tools.

The human-readable format should look like a unified git-style diff for content, and a structured "added / removed / modified" report for structural diffs. Lean on `similar`'s `TextDiff` API for content diffs; it produces unified-diff-style output natively. Use `owo-colors` for the green/red/yellow color coding if you want it to look like `git diff`.

#### Step 4.5 — Test the end-to-end versioning case

Take the completed policy you saved at the end of Stage 3. Modify one node in your library: bump its version, change its body. Compose a new completed policy from the same set of node IDs. Run `oovra compare <old> <new>`.

Verify the tool correctly identifies that exactly one node changed — by ID and by version delta — even though the rendered body text differs in many places. This is the moment the architecture pays off: the structural diff cuts through the surface noise to show the real change. Two completed policies that look totally different are revealed to differ in exactly one node's version.

#### Step 4.6 — Stop and use it for a real prompt

You now have a working composer. Take the system prompt you most often hand-edit — for [[OpenClaw]], for a Claude Code session, for [[Animus Prion]], for an [[Obsidian Vault]] agent — and break it into nodes. Use Create to scaffold them, edit their bodies, use Compose to assemble them, use Compare to diff against your old hand-written version.

Use the tool for two weeks before adding any features. The features you imagined needing will turn out to be wrong; the features you actually need will reveal themselves through use. This is the [[Architect/Builder Gap]] discipline applied: ship v0.1, eat your own cooking, only then plan v0.2.

---

## Appendix A — The Oovra Header Schema (Reference)

This appendix is the agent-facing schema definition. Ship it as `SCHEMA.md` in the Oovra repo root so agents and humans have a single canonical reference.

### Common rules

Every Oovra file begins with a TOML frontmatter block delimited by `+++` on its own line at the very top of the file, followed by a closing `+++` on its own line, followed by a blank line, followed by the Markdown body.

The frontmatter must contain a `kind` field whose value is either `"node"` or `"completed"`. This field discriminates the schema for the rest of the header.

### Node header schema

Required fields:

- `kind` — string, must be `"node"`.
- `id` — string, kebab-case, unique across the library.
- `version` — string, semantic version (e.g. `"1.0.0"`).
- `tags` — array of strings.

Optional fields:

- `name` — string, human-readable name.
- `description` — string, one-line summary.
- `requires` — array of strings, IDs of nodes this one depends on.
- `conflicts_with` — array of strings, IDs of nodes this one cannot coexist with.

Body: freeform Markdown. Wiki-style links (`[[node-id]]`) are encouraged for Obsidian compatibility but are not parsed by Oovra.

### Completed header schema

Required fields:

- `kind` — string, must be `"completed"`.
- `id` — string, kebab-case, unique across completed policies.
- `generated_at` — string, ISO 8601 timestamp.
- `render_mode` — string, identifies the renderer used (e.g. `"markdown-h2"`).
- `nodes` — array of inline tables, each with fields `id` (string) and `version` (string), in the order they were composed.

Optional fields:

- `name` — string, human-readable name.
- `description` — string, one-line summary.
- `source_library` — string, path to the library directory used at compose time.
- `overrides` — array of inline tables, each with fields `node_id` (string) and `applied_override` (string), recording any inline body overrides used during composition.

Body: the rendered Markdown produced by the composer. Treated as opaque by Oovra except by Compare in node-vs-node-equivalent mode.

---

## Appendix B — Mental Model Summary

Two file types, one format, one parser:

A **policy node** is a `.md` file with `kind = "node"` in its TOML header. Hand-authored by a human or generated by an agent. The header holds metadata; the body holds prose policy. One file per node. Lives in a `nodes/` directory.

A **completed policy** is a `.md` file with `kind = "completed"` in its TOML header. Tool-emitted by the Compose operator. The header records the composition recipe (ordered node references with versions); the body holds the rendered prompt. Self-describing: any completed policy can be Decomposed without consulting external state.

Four operators, one binary, one help screen:

- **Create** — author a new node from scratch (`--pnode`) or label an existing Markdown file as a node (`--label`).
- **Compose** — assemble a completed policy from ordered nodes. The JOIN operator. Has three output modes: full Oovra file (default), body-only text (`--text`), and re-render of an existing completed policy (`--re-render`). The `--text` mode doubles as the staged-form inspection path: rendering a single node by ID validates that the node parses without writing anything to disk.
- **Decompose** — extract the recipe from a completed policy. The SPLIT operator. Trivial because the header already contains the recipe.
- **Compare** — diff two Oovra files with kind-aware dispatch. The FORWARD-DIFF operator. Surfaces the structural similarity between completed policies that differ only in node versions.

Three audiences, one format:

Humans edit Oovra files in any text editor or in Obsidian. Agents author Oovra files by writing valid Markdown with TOML frontmatter — a format they already produce reliably. The Rust tool consumes Oovra files via one parser, one schema, one set of error messages.

---

## Appendix C — Obsidian Compatibility

Because every Oovra file is a valid Markdown file with frontmatter, the entire library is also a valid Obsidian vault. To use Oovra with Obsidian:

Open the `nodes/` directory (or whatever your library root is) as an Obsidian vault. Obsidian will index every `.md` file, render the frontmatter as properties (visible in the properties panel), and render the bodies as Markdown.

If you use `[[wiki-link]]` syntax in node bodies — for example, a refusal-policy node body might say "as established in [[role-declaration]]" — Obsidian will render those as live links you can click to navigate the library. Oovra ignores these; they're invisible to the parser. This is a free property of the format: you get visual navigation of your prompt architecture for no engineering cost.

Tags in the frontmatter (`tags = ["safety", "production"]`) become Obsidian tags, browsable from the tag pane and usable in Obsidian's search.

The graph view in Obsidian shows the dependency structure of your library: which nodes reference which, which completed policies use which nodes, which tags cluster together. This is genuine prompt architecture visualization, available for free because of the format choice.

You don't have to use Obsidian to use Oovra. Oovra has no Obsidian dependency. But the format choice means Obsidian is always an option.

---

## Follow-up Tasks (post-v0.1)

- Multi-renderer support: add render modes for Claude-style XML-tagged output (`<role>`, `<instructions>`, `<examples>`), OpenAI-style Markdown, plain text.
- Semver range matching on version pins (`^1.0`, `>=1.2, <2.0`).
- Dependency resolution: when a node declares `requires`, automatically include those dependencies in Compose unless explicitly excluded; produce topological order; detect cycles and conflicts.
- The `bundle` kind: a third file type that groups related nodes for distribution and reuse.
- Library-wide audits: "which nodes are unused across all my completed policies?" (the UNIQUE operator from your sheet, lifted to library scope), "which nodes appear in 80%+ of policies and should perhaps be defaults?"
- TUI for browsing the library with `ratatui` (good Rust learning project).
- Round-trip parsing of arbitrary existing prompts back into nodes (the hard SPLIT direction — heuristic-based, requires NLP-light boundary detection).
- Optional integration with [[Carbide]]'s string operators if you want Oovra to share a JOIN/SPLIT/UNION primitive layer with your other Rust string tooling.
- An `oovra serve` mode that exposes Oovra operations as an HTTP API for agents to call remotely, returning JSON-formatted Compare reports and rendered Compose output.
- An Obsidian plugin that wraps Oovra operations as commands inside the Obsidian UI, so you can Compose and Compare without leaving the editor.
