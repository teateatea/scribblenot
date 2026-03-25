# Plan: Add Last-Phase Field to PreCompact Hook Entry

**Task**: #32 - Add PreCompact hook to log compact events with timestamp during pathfinder missions

## Context

The pre-compact-mission-log.sh hook appends a `## PreCompact Event` block to the active MISSION-LOG, but does not record where in the mission workflow the compact occurred. Adding a `Last-Phase` field -- the most recent `##` section heading before the PreCompact Event block -- gives post-mission reviewers an immediate anchor for which mission phase was active at the time of the compact.

## Approach

Before appending the PreCompact Event block, grep the log file for all `## ` headings, exclude any line that already contains "PreCompact Event" (to avoid self-referencing a prior event entry), take the last match, strip the `## ` prefix, and store the result in `LAST_PHASE`. If no heading is found, default to `(unknown)`. Append `- Last-Phase: ${LAST_PHASE}` as a new line inside the existing append block.

## Critical Files

- `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` -- the only file modified (lines 21-29 are the timestamp + append block)

## Reuse

No external utilities; pure bash using `grep`, `tail`, and `sed` already available in the hook's execution environment.

## Steps

1. After the `TIMESTAMP` assignment (line 21), add the `LAST_PHASE` extraction block:

```diff
 TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
+
+LAST_PHASE="$(grep '^## ' "$LOG_FILE" | grep -v 'PreCompact Event' | tail -1 | sed 's/^## //')"
+if [ -z "$LAST_PHASE" ]; then
+  LAST_PHASE="(unknown)"
+fi
```

2. Add `- Last-Phase: ${LAST_PHASE}` to the appended block:

```diff
 {
   echo ""
   echo "## PreCompact Event"
   echo ""
   echo "- Timestamp: ${TIMESTAMP}"
+  echo "- Last-Phase: ${LAST_PHASE}"
   echo "- Note: PreCompact hook fired; review continuity from this point forward."
 } >> "$LOG_FILE"
```

## Verification

### Manual tests

1. In the scribblenot project directory (where `MISSION-PERMISSIONS.json` is present), add a test `## Some Phase` heading to `MISSION-LOG-active.md`.
2. Run the hook directly: `bash /c/Users/solar/.claude/hooks/pre-compact-mission-log.sh`
3. Open `MISSION-LOG-active.md` and confirm the appended block contains:
   - `- Last-Phase: Some Phase`
4. Run the hook again (now a prior `## PreCompact Event` heading exists) and confirm `Last-Phase` still reflects `Some Phase`, not `PreCompact Event`.
5. Run the hook against a log file containing only a `## PreCompact Event` heading (no other `##` lines) and confirm `- Last-Phase: (unknown)` is emitted.

### Automated tests

- Shell integration test: create a temp file with known `## ` headings, invoke the hook with `PWD` pointing to a directory containing a mock `MISSION-PERMISSIONS.json` and the temp file as the log, then assert the appended block contains the expected `Last-Phase` value using `grep -q`.

## Changelog

### Review - 2026-03-25
- Nit #1: Fixed path style in Verification step 2 from `C:/Users/solar/...` to `/c/Users/solar/...` for correct Git Bash execution.
