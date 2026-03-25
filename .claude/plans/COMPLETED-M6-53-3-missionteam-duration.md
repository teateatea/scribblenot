## Task

#53 sub-task 3 - Add estimated duration display near MT-1 start in pathfinder-mission-team/SKILL.md

## Context

When a mission starts, the Commander already computes T (sum of D scores) in MT-1 step 2. There is no step that converts T into a human-readable time estimate before the loop begins. Adding one gives the user an upfront sense of how long the mission will take, and embeds it permanently in the MISSION-LOG for later reference.

## Approach

After T is computed in MT-1 step 2, add a new step 2b that computes `ESTIMATED_DURATION = round(T * 0.43)` (minutes) and writes the line `Estimated-Duration: ~<X> min (T x 0.43)` into the `## Mission` header block of the MISSION-LOG template in step 5.

No new file, no new helper function - just two small text additions inside MT-1.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - lines 22-85 (MT-1 section)

## Reuse

- T is already computed on line 22 (`Compute T = sum of D scores`). Reuse it directly.

## Steps

1. In MT-1, after step 2a (permissions check), insert a new step **2b**:

```
2b. Compute `ESTIMATED_DURATION = round(T * 0.43)` (minutes).
```

2. In the MISSION-LOG template inside MT-1 step 5, add the `Estimated-Duration` line to the `## Mission` block, immediately after `Difficulty`:

```diff
 - Difficulty: 0/<T>
+- Estimated-Duration: ~<ESTIMATED_DURATION> min (T x 0.43)
```

The full updated `## Mission` block becomes:

```markdown
## Mission
- Slug: <MISSION_SLUG>
- Date: <date portion of START_TIME, i.e. the first 10 characters>
- Start-Time: <START_TIME>
- Tasks: <comma-separated list with initial priorities>
- Difficulty: 0/<T>
- Estimated-Duration: ~<ESTIMATED_DURATION> min (T x 0.43)
```

## Verification

### Manual tests

- Invoke `/pathfinder-mission-team` with a known task list (e.g. one task with D:5).
- Open the generated `MISSION-LOG-*.md` and confirm the `## Mission` section contains `Estimated-Duration: ~2 min (T x 0.43)` (round(5 * 0.43) = 2).

### Automated tests

- Doc check: after a test run, grep the log for `Estimated-Duration:` to confirm the field is present.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Estimated-Duration`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | ESTIMATED_DURATION = round(T * 0.43)`

## Changelog

### Review - 2026-03-25
- #1: Changed `(D x 0.43)` to `(T x 0.43)` throughout (Approach, diff block, full block example, verification example) for consistency with the `T` variable already used in `Difficulty: 0/<T>` in the same MISSION-LOG template, and to match the fix applied in the companion 53-2 plan.

## Progress
- Step 1: Inserted step 2b (ESTIMATED_DURATION = round(T * 0.43)) after step 2a in MT-1
- Step 2: Added Estimated-Duration line to the Mission block template in MT-1 step 5
