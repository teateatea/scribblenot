# Plan: M5-32-1 -- Write PreCompact hook shell script

## Task
#32 -- Add PreCompact hook to log compact events with timestamp during pathfinder missions

## Context
Claude does not expose context usage to agents, so missions cannot predict when an automatic /compact will fire. A PreCompact hook can capture the exact moment by appending a timestamped entry to the active MISSION-LOG just before compaction occurs. This sub-task covers writing the shell script itself at `~/.claude/hooks/pre-compact-mission-log.sh`. Registering the hook in `~/.claude/settings.json` under the `PreCompact` event key is out of scope here and must be handled by a subsequent sub-task.

## Approach
Write a bash script that mirrors the MANIFEST-guard idiom already used in `check-mission-permissions.sh`: check for `MISSION-PERMISSIONS.json` in cwd first, exit 0 immediately if absent (not a mission context), then locate the highest-numbered `MISSION-LOG-*.md` via shell glob (falling back to `MISSION-LOG-active.md`), and append a compact event entry. Use `date -u +"%Y-%m-%dT%H:%M:%SZ"` for a real ISO 8601 timestamp rather than constructing one from the date field alone.

## Critical Files
- **New file**: `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` (to create)
- **Reference**: `C:/Users/solar/.claude/hooks/check-mission-permissions.sh` (pattern source, lines 1-25 for MANIFEST guard and log-file selection)

## Reuse
- MANIFEST guard pattern (lines 6-9 of `check-mission-permissions.sh`): check `[ ! -f "$MANIFEST" ]` then `exit 0`
- Log-file selection pattern (lines 22-25 of `check-mission-permissions.sh`): `ls "$(pwd)"/MISSION-LOG-*.md 2>/dev/null | sort | tail -1` with fallback to `MISSION-LOG-active.md`

## Steps

1. Create `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` with the following content:

```bash
#!/usr/bin/env bash
# PreCompact hook: appends a timestamped compact event entry to the active MISSION-LOG.
# Exits silently (exit 0) if MISSION-PERMISSIONS.json is not present in cwd (not a mission context).

MANIFEST="$(pwd)/MISSION-PERMISSIONS.json"
if [ ! -f "$MANIFEST" ]; then
  exit 0
fi

# Find highest-numbered mission log; fall back to MISSION-LOG-active.md
LOG_FILE="$(ls "$(pwd)"/MISSION-LOG-*.md 2>/dev/null | sort | tail -1)"
if [ -z "$LOG_FILE" ]; then
  LOG_FILE="$(pwd)/MISSION-LOG-active.md"
fi

# No log file present at all - no-op
if [ ! -f "$LOG_FILE" ]; then
  exit 0
fi

TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
  echo ""
  echo "## PreCompact Event"
  echo ""
  echo "- Timestamp: ${TIMESTAMP}"
  echo "- Note: PreCompact hook fired; review continuity from this point forward."
} >> "$LOG_FILE"

exit 0
```

2. Make the script executable:
```
chmod +x /c/Users/solar/.claude/hooks/pre-compact-mission-log.sh
```

## Verification

### Manual tests
- Copy a MISSION-LOG-*.md and MISSION-PERMISSIONS.json to a temp directory.
- Run `bash /c/Users/solar/.claude/hooks/pre-compact-mission-log.sh` from that directory.
- Confirm a `## PreCompact Event` section with a valid ISO 8601 timestamp was appended to the log file.
- Remove MISSION-PERMISSIONS.json from the temp directory; re-run the script.
- Confirm the log file is unchanged (silent exit when no manifest).
- Remove the MISSION-LOG-*.md files but keep MISSION-PERMISSIONS.json; re-run the script.
- Confirm no error is produced and no file is created (no-op when log absent).

### Automated tests
- A bash integration test that creates temp fixtures (manifest + numbered log), invokes the script, and asserts the appended section contains `## PreCompact Event` and a timestamp matching the pattern `[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z`.
- A second assertion that running without a manifest produces no output and exits 0 without modifying any file.

## Prefect Report

### Pass 2 - 2026-03-25

**Minor**

1. `PROJECT-FOUNDATION.md:8` -- The FOUNDATION requires the entry to record "the current mission phase or task if determinable." The script appends only a fixed static note (`"PreCompact hook fired; review continuity from this point forward."`) with no attempt to capture or approximate the current phase. The plan's Approach and Steps sections do not acknowledge this requirement or explain why it is infeasible from a shell hook. The plan should either (a) add logic to extract the last task/phase heading from the log file and include it in the appended entry, or (b) explicitly state in the Approach that determining the current phase from a shell hook is not feasible (the hook has no access to Claude's internal state), and justify treating it as out of scope for this sub-task.

**Nit**

2. `plan:60,67` -- Path style is inconsistent. Critical Files (line 13-14) uses `C:/Users/solar/...` (Windows-style), while the `chmod` command in Step 2 (line 60) and the Verification run commands (lines 67-68) use `/c/Users/solar/...` (Git Bash-style). Both work in Git Bash, but the inconsistency is confusing. Prefer one style throughout the plan; the `C:/` style used in Critical Files is the project standard.

## Changelog

### Review - 2026-03-25
- #1: Added scope note to Context clarifying that hook registration in settings.json is out of scope for this sub-task and must be handled by a subsequent sub-task.

### Review #2 - 2026-03-25
- #1 (minor): Changed section header in appended log entry from `## Compact Event` to `## PreCompact Event` and updated the note text to use "PreCompact" -- aligns with PROJECT-FOUNDATION.md requirement that the entry record the event type ("PreCompact") explicitly.

### Prefect Pass - 2026-03-25
- Fixed stale `## Compact Event` reference in Manual tests verification step (line 68) to `## PreCompact Event`.
- Fixed stale `## Compact Event` reference in Automated tests assertion (line 75) to `## PreCompact Event`.
