# Plan: 5-tdd-mt3c-resolver

## Task
#5 - Add TDD-feasibility check to Decomposer for event-loop and TUI code

## Context
The MT-3c sub-task iteration preamble already contains a partial resolver note for `<TEST_RUNNER>`. It correctly explains how to pick between a per-sub-task `test_runner` field and the top-level `default_test_runner`. However, it only says that `"none"` skips the Red phase (step 1). Step 4 - "Verify TDD tests pass" - is a second TDD phase that should also be skipped when `test_runner` is `"none"`, but the resolver note does not say this. Step 4 already has its own conditional ("If TDD tests were written (not 'no tests')"), but the resolver preamble is where the definitive `"none"` semantics are declared, and it is currently silent on step 4.

## Approach
Extend the existing resolver note sentence in MT-3c to state that `"none"` skips both the Red phase (step 1) and the Verify TDD tests pass phase (step 4). No new paragraph is needed; the existing sentence is extended in-place.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - line 147 (the resolver note sentence)

## Reuse
None - this is a single-sentence doc edit.

## Steps

1. Edit the resolver note on line 147 of `SKILL.md`. Change:

```
- A `test_runner` value of `"none"` means skip the Red phase entirely for this sub-task and proceed directly to the Plan-review loop.
+ A `test_runner` value of `"none"` means skip the Red phase (step 1) and the Verify TDD tests pass phase (step 4) entirely for this sub-task; proceed directly to the Plan-review loop after resolving `<TEST_RUNNER>`.
```

The full resolver paragraph after the edit should read:

```
When iterating sub-tasks, resolve `<TEST_RUNNER>` as follows: if the sub-task JSON object has a `test_runner` field, use that value; otherwise use the top-level `default_test_runner` value. A `test_runner` value of `"none"` means skip the Red phase (step 1) and the Verify TDD tests pass phase (step 4) entirely for this sub-task; proceed directly to the Plan-review loop after resolving `<TEST_RUNNER>`.
```

## Verification

### Manual tests
- Read `SKILL.md` lines 146-148 and confirm the resolver note mentions both step 1 and step 4.
- Confirm step 4 ("Verify TDD tests pass") still has its own conditional guard ("If TDD tests were written (not 'no tests')") as a belt-and-suspenders check.

### Automated tests
None applicable - doc-only change.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | skip the Red phase (step 1) and the Verify TDD tests pass phase (step 4) entirely`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | means skip the Red phase entirely for this sub-task and proceed directly to the Plan-review loop`

## Changelog

### Review - 2026-03-25
- Reviewer #1: No issues found. Diff applies cleanly to SKILL.md line 147; step 4 conditional guard at line 256 already handles the "no tests" case as noted in the verification section. Plan approved as-is.

## Progress
- Step 1: Extended resolver note on SKILL.md line 147 to mention both step 1 (Red phase) and step 4 (Verify TDD tests pass) are skipped when test_runner is "none"
