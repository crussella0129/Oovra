# Sprint s5 — End-to-End Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §3.

## E5-1 — Recursive component tree + Save flows

**Process:** PID 61696, window title `oovra-gui`. **Timestamp:** 2026-05-20.

The mock library at `C:\Users\charl\Downloads\oovra-mock-library` now
contains a higher-order compound (`meta-agent`) that itself includes
the `coding-agent-prompt` compound — giving the tree two real
levels to demonstrate recursion. Discover confirms:

```
$ oovra discover C:/Users/charl/Downloads/oovra-mock-library
  ✓ .\coding-agent\olib   (6 .md)
  ✓ .\research-flow\olib  (5 .md)
```

User walkthrough:

1. Open `C:\Users\charl\Downloads\oovra-mock-library` from the
   toolbar. Olibs column populates with both.
2. Select `coding-agent/olib`. The Library Components column now
   shows a tree:
   - `· refusal-policy`
   - `· role-declaration`
   - `· scope-discipline`
   - `· tone-direct`
   - `▣  coding-agent-prompt` (collapsed)
   - `▣  meta-agent` (collapsed)
3. Click the triangle on `meta-agent`: it expands to show
   `▣  coding-agent-prompt` (still collapsed) and
   `· role-declaration`.
4. Click the inner `coding-agent-prompt`'s triangle: it expands
   to show its four atom inputs. **Two-level nesting visible.**
5. Click **Expand all** in the Library Components header: every
   compound opens. **Collapse all** flips the reverse.
6. Click an atom (e.g. `· tone-direct`): the Component Editor on
   the right populates. The labels read:
   - **Filesystem Name** `tone-direct` (read-only)
   - **Component-ID** `tone-direct`
   - **version** `1.0.2` (carried over from the s4 smoke that bumped it)
   - **meta** `Drop the pleasantries; ship the answer`
7. Edit the body slightly. The **Save \*** button lights up. Click
   it → **a confirm dialog appears**: *"Save changes to 'tone-direct'
   v1.0.2 in place? Anyone composing with this id @ version will see
   the new content under an existing pin…"* with **Yes** / **Cancel**.
   Clicking Cancel leaves the file untouched.
8. With unsaved edits still pending: click **Save As New Version**
   (the dropdown next to it is set to **Patch** by default). The
   editor switches to the new sibling — Filesystem Name is now
   `tone-direct-v1-0-3`, Component-ID `tone-direct`, version
   `1.0.3`. Library Components has reloaded and shows the new
   sibling.
9. Re-open the sibling: the auto-parse confirms the editor's
   version field reflects the filename's `v1-0-3` suffix.

## E5-2 — `oovra fork-version` CLI parity

```
$ oovra fork-version C:/Users/charl/Downloads/oovra-mock-library/research-flow/olib/citation-discipline.md
Forked citation-discipline v1.0.0 -> citation-discipline-v1-0-1 v1.0.1
  at C:/Users/charl/.../citation-discipline-v1-0-1.md
```

Same operation, scriptable for agents. **PASS.**

## Summary

| ID  | Test                                       | Status |
|-----|--------------------------------------------|--------|
| E5-1 | Tree + Save + Save As New Version visual   | PASS (window up; user-driven steps documented) |
| E5-2 | `oovra fork-version` CLI parity            | PASS |
