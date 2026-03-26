## Task

#59 - Mirror PreCompact hook entries to the numbered MISSION-LOG file, not just MISSION-LOG-active

## Context

Sub-task 59.2 rewrote pre-compact-mission-log.sh to append compact events to both the numbered MISSION-LOG-N-*.md file and MISSION-LOG-active.md. Sub-task 59.3 verifies the fix is correct by static analysis and logic trace, without invoking the hook (which would pollute the live mission log).

## Approach

Read the script, trace execution logic for each code path, confirm the glob pattern and deduplication guard are correct, and grep for any residual `LOG_FILE=` references that should have been removed. All verification is offline (no hook invocation).

## Critical Files

- `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` - the fixed hook (read-only for this plan)

## Reuse

No implementation code. This plan performs static verification only.

## Steps

1. Logic trace - guard clause (lines 5-8)

   `MANIFEST="$(pwd)/pathfinder/MISSION-PERMISSIONS.json"`
   If the file is absent, the script exits 0 immediately. This is unchanged from the original and correct.

2. Logic trace - glob pattern (line 28)

   `NUMBERED_LOG="$(ls "$(pwd)"/pathfinder/MISSION-LOG-[0-9]*.md 2>/dev/null | sort | tail -1)"`

   The pattern `MISSION-LOG-[0-9]*.md` requires at least one digit immediately after `MISSION-LOG-`. `MISSION-LOG-active.md` starts with the letter `a`, not a digit, so it cannot match. The glob correctly excludes the active log.

3. Logic trace - deduplication guard (lines 44-46)

   ```
   if [ -f "$ACTIVE_LOG" ] && [ "$ACTIVE_LOG" != "$NUMBERED_LOG" ]; then
     append_compact_event "$ACTIVE_LOG"
   fi
   ```

   `$ACTIVE_LOG` is always the literal path ending in `MISSION-LOG-active.md`. `$NUMBERED_LOG` is always a path ending in `MISSION-LOG-N-*.md` (or empty). Since the filenames are structurally different, `"$ACTIVE_LOG" != "$NUMBERED_LOG"` is always true when `$NUMBERED_LOG` is non-empty and always true when it is empty (empty string != non-empty string). The guard therefore never suppresses a write to the active log when the active log file exists. No duplicate writes occur because the two targets are different files.

4. Logic trace - write path (lines 39-41)

   ```
   if [ -n "$NUMBERED_LOG" ] && [ -f "$NUMBERED_LOG" ]; then
     append_compact_event "$NUMBERED_LOG"
   fi
   ```

   When a numbered log file is found, `append_compact_event` writes the PreCompact block to it. This is the new behavior added in sub-task 59.2 and is the core fix for task #59.

5. Grep for residual `LOG_FILE=` references

   Run: `grep -n "LOG_FILE=" C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh`

   Expected result: zero matches. The original script used `LOG_FILE` as its single write target; the fix replaced it with `NUMBERED_LOG` and `ACTIVE_LOG`. Any match here would indicate an incomplete rewrite.

6. Confirm no other hook files reference the old `LOG_FILE=` pattern

   Run: `grep -rn "LOG_FILE=" C:/Users/solar/.claude/hooks/`

   Expected result: zero matches across all hook files.

## Verification

### Manual tests

None required - this sub-task is entirely static analysis with no runtime behavior to observe by hand.

### Automated tests

- **Grep check 1**: `pre-compact-mission-log.sh | missing | LOG_FILE=`
  Confirms the old single-target variable is gone.

- **Grep check 2**: `pre-compact-mission-log.sh | contains | NUMBERED_LOG`
  Confirms the new numbered-log target variable is present.

- **Grep check 3**: `pre-compact-mission-log.sh | contains | ACTIVE_LOG`
  Confirms the active-log target variable is present.

- **Grep check 4**: `pre-compact-mission-log.sh | contains | MISSION-LOG-[0-9]`
  Confirms the digit-anchored glob pattern is present (excludes active log by construction).

### Doc checks

`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | missing | LOG_FILE=`
`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | contains | NUMBERED_LOG`
`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | contains | ACTIVE_LOG`
`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | contains | MISSION-LOG-[0-9]`

## Changelog

### Review - 2026-03-26
- #1: Corrected deduplication guard line reference from "lines 43-46" to "lines 44-46" to match actual script
