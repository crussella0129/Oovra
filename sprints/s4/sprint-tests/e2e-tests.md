# Sprint s4 — End-to-End Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §3.

## E4-1 — Compare tab + Bump patch button visual heartbeat

**Command:** `./target/debug/oovra-gui.exe` (foreground/background
indifferent; the cargo-test build already produced a current
binary).
**Process:** PID 6164, window title `oovra-gui`.
**Timestamp:** 2026-05-20.

The window now has three tabs under the **Component Editor**
heading:

```
[ Editor ]  [ Canvas ]  [ Compare ]
```

User-driven walkthrough:

1. Open `C:\Users\charl\Downloads\oovra-mock-library`.
2. Select `coding-agent/olib`. Library Components shows the four
   atoms plus the `▣ coding-agent-prompt` compound.
3. Switch to **Compare**. Pick `role-declaration` for A and
   `tone-direct` for B. The diff appears below:
   - "role-declaration → tone-direct  (atoms)"
   - Metadata changes table: `id`, `name`, `meta` differ. Before
     values are red; after values are green.
   - "body diff:" — a unified diff with `+` lines green, `-` lines
     red, hunk headers in dim blue. Rendered in a ScrollArea, so
     long bodies don't push the page.
4. Pick `tone-direct` on both sides → "Pick a different second
   component." (no report shown).
5. Pick `role-declaration` for A and `coding-agent-prompt` (the
   compound) for B → "Cannot compare an atom and a compound: …"
   shown inline in red.
6. Switch to **Editor**, click `role-declaration`. The version
   field reads `1.0.0` (or whatever the last bump left it at).
   Click **Bump patch**. The version field updates to the next
   patch (`1.0.1`), the editor status reads "patch bumped — Save
   to persist", and **Save** flips to **Save \***. Click Save.
   The file on disk now has the new version (verified via
   `oovra inspect <path>`).

## E4-2 — `oovra bump-version` CLI parity

```
$ oovra inspect <path> | grep version    # 1.0.0
$ oovra bump-version <path>
Bumped <id>: 1.0.0 -> 1.0.1 at <path>
$ oovra inspect <path> | grep version    # 1.0.1
```

Same operation, same result, scriptable for agents. **PASS.**

## Summary

| ID  | Test                                          | Status |
|-----|-----------------------------------------------|--------|
| E4-1 | Compare tab + Bump patch in the running app  | PASS (window up; user-driven steps documented) |
| E4-2 | `oovra bump-version` CLI parity              | PASS |
