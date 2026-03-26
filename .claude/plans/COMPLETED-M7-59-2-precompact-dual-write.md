## Task

#59 - Mirror PreCompact hook entries to the numbered MISSION-LOG file, not just MISSION-LOG-active

## Context

The pre-compact-mission-log.sh hook uses OR logic: it writes the PreCompact event to the highest-numbered MISSION-LOG-N-*.md if one exists, otherwise it falls back to MISSION-LOG-active.md. When a numbered log is active, MISSION-LOG-active.md receives no entry. This means compact events are silently missing from the active log, breaking continuity for anyone reading it during a live mission.

## Approach

Refactor the append block into a reusable shell function `append_compact_event`. After detecting the highest-numbered log, call the function for that log unconditionally, then also call it for MISSION-LOG-active.md if (a) it exists and (b) it is a different file than the numbered log. If no numbered log exists, call the function only for MISSION-LOG-active.md (preserving the current fallback behavior).

## Critical Files

- `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` - entire file; the append block is on lines 28-35; the log-selection logic is on lines 11-14

## Reuse

- Existing `grep`/`sed` pattern on line 23 for extracting `LAST_PHASE` - reuse as-is; it is called once per target file inside the new function
- Existing manifest guard (lines 5-8) - unchanged
- Existing no-op guard (lines 17-19) - adapted; at least one writable target must exist

## Steps

1. Insert the `append_compact_event` function definition immediately after the manifest guard (after line 8, before the log-selection block on line 10). The function accepts a single argument (log file path) and writes the PreCompact block to it, extracting LAST_PHASE from the target file inside the function body.

```diff
 if [ ! -f "$MANIFEST" ]; then
   exit 0
 fi
+
+append_compact_event() {
+  local target="$1"
+  local last_phase
+  last_phase="$(grep '^## ' "$target" | grep -v 'PreCompact Event' | tail -1 | sed 's/^## //')"
+  if [ -z "$last_phase" ]; then
+    last_phase="(unknown)"
+  fi
+  {
+    echo ""
+    echo "## PreCompact Event"
+    echo ""
+    echo "- Timestamp: ${TIMESTAMP}"
+    echo "- Last-Phase: ${last_phase}"
+    echo "- Note: PreCompact hook fired; review continuity from this point forward."
+  } >> "$target"
+}
+
 # Find highest-numbered mission log; fall back to MISSION-LOG-active.md
```

2. Replace the log-selection block (lines 10-19) with AND logic: locate the numbered log, set `ACTIVE_LOG` to MISSION-LOG-active.md, then decide which targets to write.

```diff
-# Find highest-numbered mission log; fall back to MISSION-LOG-active.md
-LOG_FILE="$(ls "$(pwd)"/pathfinder/MISSION-LOG-*.md 2>/dev/null | sort | tail -1)"
-if [ -z "$LOG_FILE" ]; then
-  LOG_FILE="$(pwd)/pathfinder/MISSION-LOG-active.md"
-fi
-
-# No log file present at all - no-op
-if [ ! -f "$LOG_FILE" ]; then
-  exit 0
-fi
+# Find highest-numbered mission log (MISSION-LOG-N-*.md pattern)
+NUMBERED_LOG="$(ls "$(pwd)"/pathfinder/MISSION-LOG-[0-9]*.md 2>/dev/null | sort | tail -1)"
+ACTIVE_LOG="$(pwd)/pathfinder/MISSION-LOG-active.md"
+
+# Require at least one writable target
+if [ -z "$NUMBERED_LOG" ] && [ ! -f "$ACTIVE_LOG" ]; then
+  exit 0
+fi
```

3. After the `TIMESTAMP` line, call `append_compact_event` for each applicable target.

```diff
 TIMESTAMP="$(date +"%Y-%m-%dT%H:%M:%S")"
+
+# Write to numbered log if present
+if [ -n "$NUMBERED_LOG" ] && [ -f "$NUMBERED_LOG" ]; then
+  append_compact_event "$NUMBERED_LOG"
+fi
+
+# Write to active log if it exists and is a different file than the numbered log
+if [ -f "$ACTIVE_LOG" ] && [ "$ACTIVE_LOG" != "$NUMBERED_LOG" ]; then
+  append_compact_event "$ACTIVE_LOG"
+fi
```

4. Remove the now-standalone LAST_PHASE block and original append block (lines 23-35 in the original file, now immediately following the TIMESTAMP line after steps 1-3 are applied).

```diff
-LAST_PHASE="$(grep '^## ' "$LOG_FILE" | grep -v 'PreCompact Event' | tail -1 | sed 's/^## //')"
-if [ -z "$LAST_PHASE" ]; then
-  LAST_PHASE="(unknown)"
-fi
-
-{
-  echo ""
-  echo "## PreCompact Event"
-  echo ""
-  echo "- Timestamp: ${TIMESTAMP}"
-  echo "- Last-Phase: ${LAST_PHASE}"
-  echo "- Note: PreCompact hook fired; review continuity from this point forward."
-} >> "$LOG_FILE"
```

## Verification

### Manual tests

1. Set up a mission context (MISSION-PERMISSIONS.json present). Create both a MISSION-LOG-4-foo.md and MISSION-LOG-active.md with at least one `## SomePhase` heading each. Trigger a compact event (run the hook directly: `bash ~/.claude/hooks/pre-compact-mission-log.sh`). Confirm a `## PreCompact Event` section appears in BOTH files.

2. Remove the numbered log (keep only MISSION-LOG-active.md). Run the hook. Confirm the PreCompact event appears in MISSION-LOG-active.md only (fallback behavior preserved).

3. Remove MISSION-LOG-active.md (keep only the numbered log). Run the hook. Confirm the PreCompact event appears in the numbered log only.

4. Remove both log files. Run the hook. Confirm it exits silently (no error, no file created).

### Automated tests

- Shell integration test (bats or plain bash): create temp pathfinder/ dir with MISSION-PERMISSIONS.json, populate both log variants, invoke the script, assert both files contain the `## PreCompact Event` marker. One test per scenario from manual tests 1-4 above.

## Changelog

### Review - 2026-03-26
- #1 (blocking): Step 1 diff relocated - function definition was placed at lines 23-35 (after TIMESTAMP and call sites), which causes a bash "command not found" error at runtime because a function must be defined before it is called. Moved insertion point to immediately after the manifest guard (after line 8), before the log-selection block. Step 4 updated to include an explicit diff removing the now-orphaned LAST_PHASE block and original append block.
