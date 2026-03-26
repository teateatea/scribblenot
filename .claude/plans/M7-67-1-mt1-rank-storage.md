## Task

#67 - Store premission rank in PRIORITY_MAP from BRIEF and use it as primary sort key in MT-2 and MT-3a

## Context

MT-1 step 2-A reads task IDs from `## Task Priority Order` in the BRIEF file but discards their list positions. PRIORITY_MAP is initialized later in MT-3 to 99 per task (or the P score from TASKS.md). This means every BRIEF-sourced task enters MT-3 at the same priority, causing MT-2 and MT-3a to fall back to TASK_LIST position ordering as the tiebreaker rather than the BRIEF's explicit rank. In practice TASK_LIST position mirrors BRIEF order for 2-A tasks, but the PRIORITY_MAP score itself remains 99 for all of them, which is misleading and prevents P-score-based filtering or logging from reflecting the user-reviewed sequence. Assigning each task an initial PRIORITY_MAP score of `100 - position` (99 for first, 98 for second, etc.) in 2-A makes the logged and computed priority scores reflect the user-approved BRIEF sequence while preserving the higher-is-better PRIORITY_MAP convention.

## Approach

After the BRIEF extraction loop in MT-1 2-A collects task IDs into TASK_LIST, also build an initial PRIORITY_MAP entry for each task using the formula `100 - position` (where position is 1-based). This preserves the higher-is-better PRIORITY_MAP convention: the first BRIEF task gets score 99 and is selected first by MT-3a. Tasks added via 2-B (token list) that have no BRIEF position will still default to 99 when PRIORITY_MAP is formally used in MT-3. The MT-3 initialization clause already reads "initialized to 99 per task, or the P score from TASKS.md if a [P:N ...] annotation is present" - add a third clause: "or the score assigned in MT-1 2-A if the task came from a BRIEF."

**Note on precedence:** BRIEF rank (source 1) takes precedence over any `[P:N ...]` annotation in TASKS.md (source 2) for tasks that appear in a BRIEF's `## Task Priority Order` section. If a task carries both a BRIEF position and a TASKS.md P score, the TASKS.md P score is silently ignored and the BRIEF rank is used instead. This is intentional: the BRIEF represents the user-reviewed execution sequence and should not be overridden by a static annotation. Developers who need a custom priority for a BRIEF-sourced task should reorder the task within the BRIEF rather than setting a `[P:N ...]` annotation.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - MT-1 step 2-A: lines 23-28 (BRIEF extraction loop)
  - MT-3 initialization clause: line 139 (PRIORITY_MAP description)

## Reuse

No new utilities needed. The extraction loop in 2-A already iterates task IDs in file order; the score is computed as `100 - loop_counter` (where loop_counter starts at 1).

## Steps

1. In MT-1 step 2-A, after the sentence "These task IDs become TASK_LIST, in the order they appear in the file.", add a sentence that initializes PRIORITY_MAP scores from the list positions:

```
- As task IDs are extracted, assign each task an initial PRIORITY_MAP score using the formula
  `100 - position` (pre-populates the value that MT-3 initialization will read as source (1)):
  PRIORITY_MAP[task_id] = 100 - position (99 for the first entry, 98 for the second, etc.).
  This preserves the higher-is-better PRIORITY_MAP convention: the first BRIEF task gets the
  highest score and will be selected first by MT-3a when priorities are otherwise equal.
```

2. In MT-3's PRIORITY_MAP initialization clause (line 139), extend the initialization description to include the BRIEF-rank case as the highest-precedence source:

```diff
- initialized to 99 per task, or the P score from TASKS.md if a `[P:N ...]` annotation is present
+ initialized from the first matching source: (1) `100 - position` where position is the 1-based
+ list position assigned in MT-1 2-A if the task came from a BRIEF (first task = 99, second = 98,
+ etc.), (2) the P score from TASKS.md if a `[P:N ...]` annotation is present, or (3) 99 as the
+ default
```

3. Verify that the 2-B branch (token list parse) has no rank-assignment step, so token-list tasks correctly fall through to source (2) or (3) at MT-3 initialization.

**Note on MT-2:** MT-2 reorders TASK_QUEUE by "highest priority first, then by position in TASK_LIST" - it does not read PRIORITY_MAP directly. For 2-A tasks, TASK_LIST position already mirrors BRIEF order, so MT-2's tiebreaker produces the correct ordering without code changes. The PRIORITY_MAP score written in Step 1 is consumed by MT-3a (which reads PRIORITY_MAP scores) and by the mission log (Task Status priority column), making the logged values meaningful. No MT-2 change is required.

## Verification

### Manual tests

- Run the skill with a BRIEF file whose `## Task Priority Order` lists tasks in a specific sequence (e.g. #72 first, #34 second). Confirm the mission log's Task Status table shows #72 with a higher priority score than #34 (higher score = executed first), and that execution proceeds in BRIEF order when D scores are equal.
- Run the skill with a raw token list (`#34 #71 #72`). Confirm those tasks still initialize to 99 (or their TASKS.md P score) without any BRIEF-rank override.

### Automated tests

- Unit test (shell/Python script): parse a mock BRIEF with a 3-entry `## Task Priority Order` section, simulate the 2-A extraction, and assert PRIORITY_MAP equals `{"#72": 99, "#34": 98, "#71": 97}`.
- Integration smoke test: invoke the skill against a fixture BRIEF and grep the mission log for the Task Status table; assert row order matches the BRIEF's Priority Order sequence.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | 100 - position`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | the task came from a BRIEF`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | initialized to 99 per task, or the P score from TASKS.md if a`

## Changelog

### Review - 2026-03-26
- #1: Context: corrected "fall back to D-score ordering" - actual fallback is TASK_LIST position ordering; D-scores are not used as a sort key in MT-2 or MT-3a
- #2: Steps: added Note on MT-2 clarifying why no MT-2 code change is needed (TASK_LIST position already mirrors BRIEF order; PRIORITY_MAP rank is consumed by MT-3a and the mission log)

### Review - 2026-03-26
- nit: Step 1 prose: clarified that the assignment pre-populates the value MT-3 initialization reads as source (1), removing ambiguity about when PRIORITY_MAP is formally declared vs. when values are written

### Review - 2026-03-26
- #1 (blocking): Step 1 proposed text said "as its PRIORITY_MAP rank" but doc check (line 60) expected "as its initial PRIORITY_MAP rank" - mismatch would cause doc check to fail post-implementation; added "initial" to Step 1 text to align with the doc check string

## Prefect-1 Report

### Blocking

**#1 (blocking):** Score direction inversion - SKILL.md:399, plan Steps 1-2, Verification

PRIORITY_MAP uses a higher-is-better convention: tasks start at 99 and are reduced toward 0 on failure; 0 = abandoned. MT-3a selects the task with the highest PRIORITY_MAP score. The plan proposed `PRIORITY_MAP[task_id] = position` (1 for first, 2 for second...), which would assign near-zero scores to BRIEF tasks, causing MT-3a to execute them in REVERSE BRIEF order (last task = highest score = first executed). The fix replaces the formula with `100 - position` (first task = 99, second = 98, etc.), which preserves the higher-is-better convention and causes MT-3a to pick the first BRIEF task first.

Affected locations fixed:
- `plan:7` (Context): updated description of the formula
- `plan:11` (Approach): updated formula and added higher-is-better explanation
- `plan:21` (Reuse): updated loop counter description
- `plan:Step 1 code block`: changed `PRIORITY_MAP[task_id] = position` to `100 - position` with explanation
- `plan:Step 2 diff`: updated new lines to reflect `100 - position` language
- `plan:Step 2 Note on MT-2`: "rank" -> "score"
- `plan:Verification manual test`: changed "lower priority number" and "lower number = higher priority rank" to "higher priority score = executed first"
- `plan:Verification automated test`: changed assertion from `{"#72": 1, "#34": 2, "#71": 3}` to `{"#72": 99, "#34": 98, "#71": 97}`
- `plan:Doc check line 1`: updated contains-check from `1-based position in the list as its initial PRIORITY_MAP rank` to `100 - position` to match the new Step 1 text

## Prefect-2 Report

### Minor

**#1 (minor):** Undocumented behavior - explicit `[P:N ...]` annotations in TASKS.md are silently overridden for BRIEF-sourced tasks - plan Steps 1-2, SKILL.md:139

The new priority initialization order (source 1 = BRIEF rank, source 2 = TASKS.md P score, source 3 = 99) means that a task carrying a `[P:N ...]` annotation in TASKS.md will have that annotation ignored if it also appears in the BRIEF's `## Task Priority Order`. Neither the Context, Approach, nor Steps sections mention this behavior change. A developer reading only the Approach section would not know that explicit P scores are lower-precedence than BRIEF rank for BRIEF tasks. The plan should add a note - either in the Approach or as an addendum to Step 2 - stating: "Note: for tasks that appear in both a BRIEF and carry a `[P:N ...]` annotation in TASKS.md, the BRIEF rank (source 1) takes precedence and the TASKS.md P score is ignored."

## Changelog

### Review - 2026-03-26
- #1 (blocking): Replaced ascending position scoring (1, 2, 3...) with descending formula `100 - position` (99, 98, 97...) throughout plan to match PRIORITY_MAP's higher-is-better convention; ascending scores would have caused MT-3a to execute BRIEF tasks in reverse order

### Review - 2026-03-26
- #1 (minor): Approach: added explicit note documenting that BRIEF rank (source 1) takes precedence over any TASKS.md `[P:N ...]` annotation (source 2) for BRIEF-sourced tasks; the override behavior was previously undocumented and could mislead developers expecting P scores to influence BRIEF-task ordering

## Prefect-3 Report

### Nit

**#1 (nit):** Duplicate `## Changelog` section - plan lines 69 and 108

The plan contains two separate `## Changelog` headings. The first (line 69) holds three pre-Prefect review entries. The Prefect-1 and Prefect-2 Report sections are inserted between them, and then a second `## Changelog` section (line 108) holds the two Prefect changelog entries. There should be a single `## Changelog` section; the second heading and its entries should be merged into the first, with the Prefect Report sections appearing after the unified Changelog.

## Progress

- Step 1: Added PRIORITY_MAP score initialization (100 - position) to MT-1 2-A extraction loop in SKILL.md
- Step 2: Updated MT-3 PRIORITY_MAP initialization clause to use source-precedence list (BRIEF rank, TASKS.md P score, or 99 default)
- Step 3: Verified 2-B branch (token list parse) has no rank-assignment step; token-list tasks correctly fall through to source (2) or (3)
