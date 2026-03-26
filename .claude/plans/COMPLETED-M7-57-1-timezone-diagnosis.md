## Task

#57 - Fix M6 Start-Time recorded ~4 hours ahead of actual local time

## Context

Mission 6 logs show `Start-Time: T19:06` but the user confirmed the mission started around 15:12 Eastern time. The `pathfinder-mission-team` skill and `pre-compact-mission-log.sh` hook both use `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` to generate timestamps.

On this Windows system (Git Bash / MSYS2 environment), the shell already runs in Eastern time. `TZ=America/Toronto` does not convert from UTC to Toronto - it converts from the shell's already-local time *back* through the TZ mechanism, producing UTC output instead of local time. The result is a timestamp approximately 4 hours ahead of actual Eastern time during EDT (UTC-4).

**Confirmed bash test results (run 2026-03-26 at ~03:18 Eastern):**

| Command | Output | Interpretation |
|---|---|---|
| `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` | `2026-03-26T07:18:51` | UTC (wrong - 4h ahead of local) |
| `date +"%Y-%m-%dT%H:%M:%S"` | `2026-03-26T03:18:51` | Correct local Eastern time |
| `date -u +"%Y-%m-%dT%H:%M:%S"` | `2026-03-26T07:18:51` | UTC (matches TZ=America/Toronto output) |

The `TZ=America/Toronto` prefix, rather than producing Toronto local time, produces UTC on this system. Plain `date` already returns correct local time (Eastern). This confirms the double-offset hypothesis: the shell is already in Eastern time, and setting `TZ=America/Toronto` overrides it to UTC.

## Approach

This sub-task is documentation-only. The diagnosis is complete. The fix (replacing `TZ=America/Toronto date` with plain `date` in the skill and hook) is scoped to a separate sub-task. No file edits are required here.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - contains all `TZ=America/Toronto date` calls (lines 68, 159, 347, 375, 382, 390, 398, 444, 470, 494)
- `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` - line 21: `TIMESTAMP="$(TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S")"`

## Reuse

No code reuse required - this is a diagnostic/documentation sub-task.

## Steps

1. (Complete) Run `TZ=America/Toronto date`, `date`, and `date -u` in the scribblenot bash environment and record outputs.
2. (Complete) Grep `pathfinder-mission-team/SKILL.md` for all `TZ=America/Toronto date` occurrences and note line numbers.
3. (Complete) Grep `C:/Users/solar/.claude/hooks/` for timestamp commands.
4. (Complete) Record findings in this plan file as the diagnostic artifact.

**Root cause summary:** On this Windows/MSYS2 shell, `TZ=America/Toronto` does not anchor to Toronto local time - it produces UTC output. Plain `date` already returns correct Eastern local time. Every `TZ=America/Toronto date` call in the skill and hook produces a UTC timestamp, which during EDT (UTC-4) reads ~4 hours ahead of actual local time.

## Verification

### Manual tests

- Confirm: run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` and compare to system clock - should be ~4h ahead during EDT.
- Confirm: run `date +"%Y-%m-%dT%H:%M:%S"` and compare to system clock - should match local time.

### Automated tests

None applicable - this is a diagnostic documentation sub-task. Automated verification of the fix belongs to the implementation sub-task.

### Doc checks

`C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/plans/M7-57-1-timezone-diagnosis.md | contains | TZ=America/Toronto`
`C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/plans/M7-57-1-timezone-diagnosis.md | contains | plain \`date\` already returns correct`
