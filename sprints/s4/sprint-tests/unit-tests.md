# Sprint s4 — Unit Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §1.

## Library — `oovra::header`

Five `#[test]`s added in `src/header.rs::tests`.

| ID  | Test                                          | Status |
|-----|-----------------------------------------------|--------|
| U4-1 | `bump_version_patch` (1.2.3 → 1.2.4, 0.0.0 → 0.0.1) | PASS |
| U4-2 | `bump_version_minor_resets_patch` (1.2.3 → 1.3.0) | PASS |
| U4-3 | `bump_version_major_resets_minor_and_patch` (1.2.3 → 2.0.0) | PASS |
| U4-4 | `bump_version_strips_pre_and_build` (1.2.3-rc1+sha → 1.2.4) | PASS |
| U4-5 | `bump_version_rejects_garbage` ("not-a-version", "", "1.0" all error) | PASS |

## GUI — `oovra-gui::compare`

Four `#[test]`s in `gui/src/compare.rs::tests`.

| ID  | Test                                              | Status |
|-----|---------------------------------------------------|--------|
| U4-6 | `compare_state_clears_report_when_either_side_unset` | PASS |
| U4-7 | `compare_state_produces_content_diff_for_two_atoms` | PASS |
| U4-8 | `compare_state_errors_on_mixed_kind`              | PASS |
| —    | `compare_state_same_id_both_sides_clears`         | PASS |

## Carry-forward

s1/s2/s3 unit tests all green (3 app + 3 editor + 4 canvas + s0 smoke).

## Grand totals

- `cargo test -p oovra` — 41 lib unit (36 + 5 new bump) + 4 main +
  25 integration (24 + 1 new bump round-trip) = **70 PASS**.
- `cargo test -p oovra-gui` — 3 app + 3 editor + 4 canvas + 4
  compare = **14 PASS**.

Timestamp: 2026-05-20.
