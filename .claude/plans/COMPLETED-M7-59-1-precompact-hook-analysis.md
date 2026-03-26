## Task

#59 - Mirror PreCompact hook entries to numbered MISSION-LOG

## Context

Task #59 requires that PreCompact hook events appear in the permanent numbered MISSION-LOG-N-*.md, not only in MISSION-LOG-active.md. This sub-task (59.1) documents the current hook logic, identifies the bug, and specifies the exact fix for sub-task 59.2 to implement.

## Approach

Documentation-only analysis. No code changes in this sub-task. The findings here serve as the authoritative specification for the 59.2 implementation plan.

## Critical Files

- `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` (lines 10-35: write-target selection and append logic)
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` (lines 69, 114: numbered log creation; lines 597-600: active log truncation at mission end)

## Hook Write-Target Logic (Current)

The hook resolves its write target in two steps:

Step 1 (line 11): `LOG_FILE="$(ls "$(pwd)"/pathfinder/MISSION-LOG-*.md 2>/dev/null | sort | tail -1)"`

Step 2 (lines 12-14): if `LOG_FILE` is empty, fall back to `MISSION-LOG-active.md`.

The glob `MISSION-LOG-*.md` matches both `MISSION-LOG-7-skill-log-hardening.md` and `MISSION-LOG-active.md`. Because `sort | tail -1` picks the lexicographically last filename, and the string `"active"` sorts after any digit prefix (verified: `echo -e "active\n7-skill-log-hardening" | sort | tail -1` returns `active`), `MISSION-LOG-active.md` always wins.

Result: during an active mission, the hook always writes to `MISSION-LOG-active.md` and never to the numbered log. The fall-back branch (lines 12-14) is never reached because the glob always matches `MISSION-LOG-active.md`.

## Bug Identification

**The hook has a single write target when it should have two.**

The design intent (per task #59 and mission context) is:
- `MISSION-LOG-active.md` - rolling per-session log, truncated at mission end
- `MISSION-LOG-N-*.md` - permanent mission record, retained after mission end

PreCompact events belong in the permanent record. The hook's current sort-based selection guarantees they land only in the active log, which is truncated at mission end (SKILL.md lines 597-600). This means compact events are permanently lost from the mission record.

The bug is not in the fall-back path; it is in the primary selection: the glob should be narrowed to numbered logs only, and the hook should write to both targets.

## Exact Fix for Sub-task 59.2

Replace the single-target selection block (lines 10-14) with a two-target approach:

1. Find the numbered log using a glob that excludes `MISSION-LOG-active.md`:
   `NUMBERED_LOG="$(ls "$(pwd)"/pathfinder/MISSION-LOG-[0-9]*.md 2>/dev/null | sort | tail -1)"`

2. Always set `ACTIVE_LOG="$(pwd)/pathfinder/MISSION-LOG-active.md"`.

3. Append the PreCompact entry to `MISSION-LOG-active.md` if it exists (existing behavior, preserved).

4. If `NUMBERED_LOG` is non-empty and the file exists, also append the same entry to it.

5. If neither file exists, exit 0 (no-op, same as current behavior).

This ensures:
- Active log continues to receive entries (no regression)
- Numbered log receives entries during active missions
- If no numbered log exists (pre-mission or post-truncation), the hook degrades gracefully to active-only

## Reuse

The append block (lines 28-35 of the hook) is already correct and should be reused verbatim for both write targets. No change to the content of the PreCompact entry is needed.

## Steps

1. Read `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` in full before editing.

2. Replace the write-target selection block (lines 10-19) with the two-target approach described above. Unified diff:

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
+# Find highest-numbered mission log (digits only - excludes MISSION-LOG-active.md)
+NUMBERED_LOG="$(ls "$(pwd)"/pathfinder/MISSION-LOG-[0-9]*.md 2>/dev/null | sort | tail -1)"
+ACTIVE_LOG="$(pwd)/pathfinder/MISSION-LOG-active.md"
+
+# No log file present at all - no-op
+if [ ! -f "$ACTIVE_LOG" ] && { [ -z "$NUMBERED_LOG" ] || [ ! -f "$NUMBERED_LOG" ]; }; then
+  exit 0
+fi
```

3. Replace the single append block (lines 28-35) with a loop over both targets:

```diff
-{
-  echo ""
-  echo "## PreCompact Event"
-  echo ""
-  echo "- Timestamp: ${TIMESTAMP}"
-  echo "- Last-Phase: ${LAST_PHASE}"
-  echo "- Note: PreCompact hook fired; review continuity from this point forward."
-} >> "$LOG_FILE"
+append_entry() {
+  local target="$1"
+  if [ -f "$target" ]; then
+    {
+      echo ""
+      echo "## PreCompact Event"
+      echo ""
+      echo "- Timestamp: ${TIMESTAMP}"
+      echo "- Last-Phase: ${LAST_PHASE}"
+      echo "- Note: PreCompact hook fired; review continuity from this point forward."
+    } >> "$target"
+  fi
+}
+
+append_entry "$ACTIVE_LOG"
+if [ -n "$NUMBERED_LOG" ]; then
+  append_entry "$NUMBERED_LOG"
+fi
```

3.5. Update the `LAST_PHASE` source (lines 23-26) to prefer the numbered log if present, since it is the canonical record. Unified diff:

```diff
-LAST_PHASE="$(grep '^## ' "$LOG_FILE" | grep -v 'PreCompact Event' | tail -1 | sed 's/^## //')"
-if [ -z "$LAST_PHASE" ]; then
-  LAST_PHASE="(unknown)"
-fi
+_LAST_PHASE_SRC="${NUMBERED_LOG:-$ACTIVE_LOG}"
+LAST_PHASE="$(grep '^## ' "$_LAST_PHASE_SRC" 2>/dev/null | grep -v 'PreCompact Event' | tail -1 | sed 's/^## //')"
+if [ -z "$LAST_PHASE" ]; then
+  LAST_PHASE="(unknown)"
+fi
```

4. Verify the hook is executable (`chmod +x` is a no-op if already set; confirm with `ls -l`).

## Verification

### Manual tests

1. With mission 7 active (MISSION-LOG-7-skill-log-hardening.md present), invoke the hook directly:
   `bash C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh`
   Confirm a PreCompact Event section appears at the end of MISSION-LOG-7-skill-log-hardening.md.

2. Confirm the same entry also appears in MISSION-LOG-active.md (both targets written).

3. Temporarily rename the numbered log and re-invoke. Confirm the hook still writes to MISSION-LOG-active.md without error and exits 0.

4. Remove MISSION-LOG-active.md temporarily and re-invoke with a numbered log present. Confirm the hook writes to the numbered log and exits 0.

### Automated tests

- Shell integration test: create a temp directory with a mock MISSION-PERMISSIONS.json and two mock log files (MISSION-LOG-3-test.md and MISSION-LOG-active.md), invoke the hook, assert both files contain the PreCompact Event section.
- Edge case test: only MISSION-LOG-active.md present - assert hook exits 0 and active log contains the entry.
- Edge case test: only MISSION-LOG-7-test.md present (no active log) - assert hook exits 0 and numbered log contains the entry.

### Doc checks (pseudocode assertions - grep the file manually or via a test harness)

`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | contains | MISSION-LOG-[0-9]`
`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | contains | append_entry`
`C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh | missing | MISSION-LOG-active.md" 2>/dev/null | sort | tail -1`

## Changelog

### Review - 2026-03-26
- #1 (minor): Corrected Step 2 line range from "lines 10-35" to "lines 10-19" to match the actual diff scope.
- #2 (minor): Promoted the detached LAST_PHASE Note into explicit Step 3.5 with a unified diff for lines 23-26.
- #3 (nit): Fixed missing hyphen in diff comment: "highest numbered" -> "highest-numbered".
- #4 (nit): Labelled Doc checks section as pseudocode assertions to avoid confusion with shell syntax.
