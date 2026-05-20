# Sprint s5 — Unit Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §1.

## Library — `oovra::header`

Ten new `#[test]`s added in `src/header.rs::tests`.

| ID  | Test                                                          | Status |
|-----|---------------------------------------------------------------|--------|
| U5-1 | `parse_filename_version_strips_three_part_suffix`             | PASS |
| U5-2 | `parse_filename_version_no_suffix`                            | PASS |
| U5-3 | `parse_filename_version_handles_v_in_canonical`               | PASS |
| U5-4 | `parse_filename_version_rejects_two_part_suffix`              | PASS |
| U5-5 | `parse_filename_version_rejects_non_digit_after_v`            | PASS |
| U5-6 | `parse_filename_version_handles_multidigit_parts`             | PASS |
| —    | `parse_filename_version_prefers_trailing_suffix`              | PASS |
| U5-7 | `compose_versioned_filename_round_trips`                      | PASS |
| —    | `compose_versioned_filename_strips_pre_and_build`             | PASS |
| —    | `compose_versioned_filename_rejects_garbage`                  | PASS |

## GUI — `oovra-gui::editor`

| ID  | Test                                                          | Status |
|-----|---------------------------------------------------------------|--------|
| U5-8 | `editor_autoparse_overrides_version_from_filename_suffix`    | PASS |

## Grand totals

- `cargo test -p oovra` — **51 lib unit + 4 main unit + 26 integ = 81 PASS**
  (was 70 in s4; +10 lib unit, +1 integ).
- `cargo test -p oovra-gui` — **15 PASS** (was 14; +1 editor auto-parse).

Timestamp: 2026-05-20.
