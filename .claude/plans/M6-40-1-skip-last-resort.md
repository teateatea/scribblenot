## Task
#40 sub-task 1 - Rewrite MT-1 step 2a skip framing: declare skipping as a last resort, require detailed justification, add escalation-before-skip instruction

## Context
MT-1 step 2a currently removes tasks from TASK_LIST without any friction when no matching entry is found in MISSION-PERMISSIONS.json. The reason string written to SKIPPED_TASKS is generic ("not found in MISSION-PERMISSIONS.json approved_actions"), giving no information about what was actually checked or why the match failed. This makes silent task drops too easy and produces unhelpful audit trails.

## Approach
Edit the per-task skip bullet in step 2a of the SKILL.md to:
1. Frame skipping as a last resort, not the default.
2. Require the SKIPPED_TASKS reason string to be detailed (what entries were scanned, why none matched).
3. Add an explicit instruction to prefer "log the gap and continue" over silently dropping, and to record that no escalation path was available (since this is a zero-interaction mission loop).
The wildcard logic and overall per-task check structure are left intact.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` lines 34-36 (the skip bullet inside the per-task check)

## Reuse
No new utilities needed. Change is text-only within the SKILL.md.

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and locate the per-task skip bullet (lines 34-36):

```
    - If NO entry matches: the task is not covered by the premission run.
      - Remove it from TASK_LIST.
      - Append to SKIPPED_TASKS: `{ "task": "#N", "reason": "not found in MISSION-PERMISSIONS.json approved_actions" }`.
```

2. Replace it with the expanded version below:

```diff
-    - If NO entry matches: the task is not covered by the premission run.
-      - Remove it from TASK_LIST.
-      - Append to SKIPPED_TASKS: `{ "task": "#N", "reason": "not found in MISSION-PERMISSIONS.json approved_actions" }`.
+    - If NO entry matches: **skipping is a last resort.** Before removing the task, confirm that
+      (a) the wildcard check above was not triggered, and (b) every entry in `approved_actions`
+      was scanned for `#<taskId>` as a whole token. Only if both are true may the task be dropped.
+      Because this is a zero-interaction loop, escalation to the user is not possible at runtime;
+      record that fact explicitly so the operator can act on it after the mission.
+      - Remove the task from TASK_LIST.
+      - Append to SKIPPED_TASKS with a detailed reason:
+        `{ "task": "#N", "reason": "No matching approved_actions entry found. Scanned <K> entries; none contained '#<taskId>' as a whole token. HAS_WILDCARD_ENTRY was false. Skipping is a last resort — re-run /pathfinder-premission to authorize this task." }`.
+        (Replace `<K>` with the actual count of entries scanned and `<taskId>` with the real task ID.)
```

## Verification

### Manual tests
- Invoke `/pathfinder-mission-team` with a task ID that is present in MISSION-PERMISSIONS.json and one that is absent. Confirm the absent task appears in the Skipped Tasks section of the mission log with a reason that includes the entry count and the "last resort" note.
- Confirm the present task proceeds normally and is not logged as skipped.
- Confirm the wildcard path (an entry whose rationale has no `#<digit>` token) still bypasses filtering entirely.

### Automated tests
- A unit test could parse a synthetic MISSION-PERMISSIONS.json with N entries and a task ID not present in any rationale, then assert the generated reason string contains the entry count and the substring "last resort".
- A second case could include a wildcard entry and assert SKIPPED_TASKS remains empty.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | skipping is a last resort`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Scanned`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | "not found in MISSION-PERMISSIONS.json approved_actions"`

## Progress
- Step 1: Located per-task skip bullet at lines 34-36 in pathfinder-mission-team/SKILL.md
- Step 2: Replaced generic skip bullet with last-resort framing, dual confirmation checks, zero-interaction note, and detailed reason string including entry count and task ID

## Implementation
Complete -- 2026-03-25
