## Task

#57 - Fix M6 Start-Time recorded ~4 hours ahead of actual local time

## Context

On this Windows/MSYS2 system, `TZ=America/Toronto date` does not produce Eastern time output. Instead it produces UTC. Plain `date` (without the TZ prefix) already returns the correct local Eastern time because Windows has the system timezone set correctly. Task #43 (sub-task 1) fixed `pre-compact-mission-log.sh` line 21, but the fix was not applied to `pathfinder-mission-team/SKILL.md`. Sub-task 2 completes the fix by updating both files consistently.

The `TZ=America/Toronto ` prefix was added by task #36 to force Toronto-local timestamps. On Linux/macOS that prefix correctly overrides the process timezone at runtime. On MSYS2/Windows it silently produces UTC instead, causing all mission timestamps to appear ~4-5 hours ahead of wall-clock time.

## Approach

Remove the `TZ=America/Toronto ` prefix from every `TZ=America/Toronto date` call in `pathfinder-mission-team/SKILL.md` and `pre-compact-mission-log.sh`. No new utilities, no restructuring -- just a targeted string replacement of the prefix in each affected line. Lines 375, 382, 390, and 398 in SKILL.md contain the pattern inside inline template text (`<current TZ=America/Toronto date ...>`); these must also be updated to `<current date ...>`.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 68: `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` (START_TIME)
  - Line 159: `TZ=America/Toronto date +"%H:%M"` (NOW_HH_MM)
  - Line 347: `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` (SUBTASK_TIME)
  - Lines 375, 382, 390, 398: `<current TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S">` (enforcement warning templates)
  - Line 444: `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` (CASUALTY_TIME)
  - Line 470: `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` (END_TIME - abandonment)
  - Line 494: `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` (END_TIME - mission complete)
- `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh`
  - Line 21: `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` (TIMESTAMP)

## Reuse

No utilities to reuse. All changes are plain text replacements using the Edit tool with `replace_all: true` per file.

## Steps

1. Edit `pathfinder-mission-team/SKILL.md`: replace all occurrences of `TZ=America/Toronto date` with `date` using `replace_all: true`. This covers all 10 occurrences in one operation, including both the runnable command forms and the inline template text forms.

```
- Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"`
+ Run `date +"%Y-%m-%dT%H:%M:%S"`

- Run `TZ=America/Toronto date +"%H:%M"`
+ Run `date +"%H:%M"`

- **5. Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` and store the result as `SUBTASK_TIME`.
+ **5. Run `date +"%Y-%m-%dT%H:%M:%S"` and store the result as `SUBTASK_TIME`.

-        - Timestamp: <current TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S">
+        - Timestamp: <current date +"%Y-%m-%dT%H:%M:%S">

- Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` and store the result as `CASUALTY_TIME`.
+ Run `date +"%Y-%m-%dT%H:%M:%S"` and store the result as `CASUALTY_TIME`.

- 1. Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` and store the result as `END_TIME`.
+ 1. Run `date +"%Y-%m-%dT%H:%M:%S"` and store the result as `END_TIME`.
```

2. Edit `pre-compact-mission-log.sh`: replace the single occurrence on line 21.

```
- TIMESTAMP="$(TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S")"
+ TIMESTAMP="$(date +"%Y-%m-%dT%H:%M:%S")"
```

3. Verify with grep that no `TZ=America/Toronto date` occurrences remain in either file.

## Verification

### Manual tests

- Start a new pathfinder mission and confirm that the `Start-Time:` written to the MISSION-LOG matches the actual wall-clock time (within a minute).
- Trigger a sub-task completion and confirm the sub-task log timestamp is correct.
- If possible, trigger the pre-compact hook (or inspect a test fire) and confirm the PreCompact Event timestamp is correct.

### Automated tests

- Run `grep -c "TZ=America/Toronto" C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and confirm the result is `0`.
- Run `grep -c "TZ=America/Toronto" C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` and confirm the result is `0`.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | TZ=America/Toronto date`
`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | missing | TZ=America/Toronto`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Run \`date +"%Y-%m-%dT%H:%M:%S"\` and store the result as \`START_TIME\``
`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | contains | TIMESTAMP="$(date +"%Y-%m-%dT%H:%M:%S")"`
