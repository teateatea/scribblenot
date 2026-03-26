## Task

#60 - Add Initial and Current Estimated Completion Time fields to MISSION-LOG Task Status

## Context

The MISSION-LOG template already has placeholder tokens `<INITIAL_ETA>`, `<CURRENT_ETA>`, and `<UPDATE_TIME>` in the `## Mission` section header block, but MT-1 step 5 has no instruction telling the commander how to compute those values. At mission start, both ETA fields should be identical and derived from `START_TIME + ESTIMATED_DURATION` (where `ESTIMATED_DURATION = round(T * 0.43)` minutes). Sub-task 2 covers the initial write during MT-1 log creation; a later sub-task will handle recomputation on each new task start.

## Approach

Add a single instruction block in MT-1 step 5, immediately after `ESTIMATED_DURATION` is available (step 2b) and before the file is written, that computes `INITIAL_ETA` and sets `CURRENT_ETA = INITIAL_ETA`, `UPDATE_TIME = START_TIME` formatted as `HH:mm`. These three values are then substituted into the template when writing the MISSION-LOG file.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 62: `2b. Compute ESTIMATED_DURATION = round(T * 0.43)` - insert ETA computation here, as a new `2c` step
  - Lines 67-103: MT-1 step 5 template write block - the placeholder tokens `<INITIAL_ETA>`, `<CURRENT_ETA>`, `<UPDATE_TIME>` are already in the template at lines 79-80 and need no changes

## Reuse

- `ESTIMATED_DURATION` computed in step 2b (already present)
- `START_TIME` set in step 5 via `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` (already present)
- No new utilities needed; wall-clock addition uses simple integer minute arithmetic

## Steps

1. In SKILL.md, after step 2b (`Compute ESTIMATED_DURATION = round(T * 0.43)`) and before step 3, insert a new step `2c`:

```
- old (after line 62):

2b. Compute `ESTIMATED_DURATION = round(T * 0.43)` (minutes).

3. Derive `MISSION_SLUG`: a 2-3 word lowercase hyphenated summary of the task list (e.g. `auth-refactor-tests`).

+ new:

2b. Compute `ESTIMATED_DURATION = round(T * 0.43)` (minutes).

2c. Compute `INITIAL_ETA`: parse `START_TIME` (ISO-8601 `YYYY-MM-DDTHH:MM:SS`) to extract the wall-clock hour and minute, add `ESTIMATED_DURATION` minutes, wrap modulo 1440 (minutes in a day) if needed, and format the result as `HH:mm` (zero-padded). Set `CURRENT_ETA = INITIAL_ETA`. Set `UPDATE_TIME` = `START_TIME` formatted as `HH:mm` (the HH:MM portion of START_TIME, zero-padded).

3. Derive `MISSION_SLUG`: a 2-3 word lowercase hyphenated summary of the task list (e.g. `auth-refactor-tests`).
```

2. No template change is needed. The template at lines 79-80 already contains:
   ```
   - Initial Estimated Completion Time: <INITIAL_ETA> (Started at <START_TIME>)
   - Current Estimated Completion Time: <CURRENT_ETA> (Updated at <UPDATE_TIME>)
   ```
   When MT-1 step 5 writes the file, substitute the values computed in step 2c for `<INITIAL_ETA>`, `<CURRENT_ETA>`, and `<UPDATE_TIME>`, and the existing `<START_TIME>` substitution for the "Started at" field.

3. Update the inline comment in step 6 that references where `Prior-Auto-Accept` is appended. The current text says "immediately after the `Estimated-Duration:` line; if `Estimated-Duration:` is absent, append after `Difficulty:`". No change needed here since the ETA lines appear after `Estimated-Duration:` and the `Prior-Auto-Accept` append target remains `Estimated-Duration:`.

## Verification

### Manual tests

- Run `/pathfinder-mission-team` on a test task list. Open the generated MISSION-LOG file and confirm:
  - `Initial Estimated Completion Time:` line is present in `## Mission` with a valid `HH:mm` time and a `Started at` timestamp.
  - `Current Estimated Completion Time:` line is present with the same `HH:mm` value and `Updated at` equal to the start time's `HH:mm`.
  - The computed ETA equals `START_TIME HH:mm` plus `ESTIMATED_DURATION` minutes (verify manually with a calculator for a known T value).
- Confirm a mission with T=0 (D scores all zero) produces `INITIAL_ETA = START_TIME HH:mm` (no offset).
- Confirm a mission where adding ESTIMATED_DURATION crosses midnight (e.g., START_TIME = 23:50, ESTIMATED_DURATION = 20) produces a correct next-day wrap (00:10).

### Automated tests

- Unit test (shell or Python): given a fixed ISO-8601 START_TIME string and a known T, assert the computed INITIAL_ETA string equals the expected `HH:mm` value (covers normal case, wrap-around midnight).

## Changelog

### Review - 2026-03-26
- #1: Expanded truncated context lines in Step 1 diff block to match actual SKILL.md content (`3. Derive \`MISSION_SLUG\`:` -> full line with summary description and example).
