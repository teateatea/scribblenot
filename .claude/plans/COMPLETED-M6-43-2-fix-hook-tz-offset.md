## Task

#43 - Drop UTC offset from pathfinder timestamps, output bare local datetime

## Context

The pre-compact hook writes a timestamp to the mission log in the format `2026-03-25T15:30:00-0400`. The `-0400` UTC offset is noise for a single-user, single-machine setup. This sub-task removes `%z` from the date format string on line 21 of the hook so its output matches the bare local datetime format already used in SKILL.md.

## Approach

Single-line edit to the `date` format string: remove `%z` from `+"%Y-%m-%dT%H:%M:%S%z"`, leaving `+"%Y-%m-%dT%H:%M:%S"`.

## Critical Files

- `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` - line 21

## Reuse

No utilities to reuse; this is a one-character string deletion in a shell script.

## Steps

1. Edit line 21 of `pre-compact-mission-log.sh` to remove `%z`:

```diff
-TIMESTAMP="$(TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S%z")"
+TIMESTAMP="$(TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S")"
```

## Verification

### Manual tests

- Trigger a compact event (or simulate by running the hook directly: `bash C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` from the scribblenot project root).
- Open the mission log and confirm the new PreCompact Event entry has a timestamp in the format `2026-03-25T15:30:00` with no trailing offset.

### Automated tests

- Unit-style shell test: invoke the hook in a test environment and grep the appended line for a timestamp matching `[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}$` (anchored at end of field, no offset characters).

## Progress

- Step 1: Removed %z from date format string on line 21 of pre-compact-mission-log.sh
