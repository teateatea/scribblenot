## Task
#48 - Suppress diff windows globally when the auto-approve permission hook is active

## Context
Claude Code shows diff-view windows whenever a file is edited. When the auto-approve PermissionRequest hook (`check-mission-permissions.sh`) is active, those diffs nearly always misfire - the permission is auto-approved before the user can interact, leaving stale or empty diff popups behind. The user wants diffs suppressed permanently while that hook setup is in place, not just during missions.

Currently, `autoAcceptEdits: true` is only written transiently by the pathfinder-mission-team skill (MT-1 step 6, MT-4 step 5, MT-3f step 5) and removed when the mission ends. Outside of missions, `check-mission-permissions.sh` exits 0 immediately (no manifest present), but it is always registered as a PermissionRequest hook in `~/.claude/settings.json`. The diff problem therefore exists at all times, not just during missions.

## Research findings

**Correct settings key:** `autoAcceptEdits` (boolean, top-level field in any Claude Code settings JSON). This is confirmed by its use in the pathfinder-mission-team skill and its presence in the project's current `settings.local.json`.

**Where it already lives:** `C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/settings.local.json` currently has `"autoAcceptEdits": true` at the top level (line 27). This value was written by a prior mission run and never removed - so diffs are already suppressed for this project right now.

**Correct scope for a permanent fix:** `settings.local.json` (project-level, not committed to git). Reasons:
- The auto-approve hook is registered in `~/.claude/settings.json` (user-global), but `autoAcceptEdits` in the global settings.json would suppress diffs for ALL Claude Code projects, including ones where the user might want diffs.
- `settings.local.json` is project-scoped and gitignored by default. Placing `autoAcceptEdits: true` there suppresses diffs only in the scribblenot project, which is exactly the project where the auto-approve hook is active.
- The mission-team skill already correctly targets `settings.local.json` for transient suppress/restore. A permanent value in the same file means missions do a no-op write (key already true) and a no-op restore (key stays true after removal... wait - the restore step removes the key). See edge case below.

**Edge case - mission restore step:** MT-4 step 5 and MT-3f step 5 remove `autoAcceptEdits` from `settings.local.json` after a mission ends. If task #48 makes this key permanent in `settings.local.json`, the restore step will delete it, reverting to the pre-task-#48 state (diffs re-enabled). Sub-task 2 must update the mission restore steps to check whether `autoAcceptEdits` was already present before the mission started, and only remove it if it was not pre-existing.

**The auto-approve hook and diffs:** `check-mission-permissions.sh` is a PermissionRequest hook - it fires when Claude Code asks the user to approve a tool. `autoAcceptEdits` suppresses the diff popup that accompanies Edit/Write permission requests. These are orthogonal mechanisms: the hook auto-approves the underlying permission; `autoAcceptEdits` suppresses the visual diff window. Both are needed for smooth autonomous operation.

## Approach
Sub-task 1 is research-only. Sub-task 2 should:
1. Ensure `"autoAcceptEdits": true` is permanently present in `settings.local.json` (it already is, but the value should be documented as intentional, not accidental).
2. Update the mission-team skill's restore steps (MT-4 step 5 and MT-3f step 5) to only remove `autoAcceptEdits` if it was not already set before the mission began, preserving the permanent value when task #48 is in effect.

## Critical Files
- `C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/settings.local.json` - already has `"autoAcceptEdits": true`; no change needed for the permanent value itself
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - MT-1 step 6 (write autoAcceptEdits), MT-4 step 5 and MT-3f step 5 (restore/remove it) - these restore steps need a guard

## Reuse
- Existing `autoAcceptEdits` key and the pattern already established by pathfinder-mission-team skill
- No new utilities needed; change is limited to SKILL.md wording for the restore steps

## Steps
1. Confirm `settings.local.json` already contains `"autoAcceptEdits": true` permanently (verified: it does).
2. In `pathfinder-mission-team/SKILL.md`, update MT-1 step 6 to read the pre-existing value of `autoAcceptEdits` before writing. Set `PRIOR_AUTO_ACCEPT` = `true` if the key exists and is `true` in `settings.local.json`, otherwise `false`. Append `- Prior-Auto-Accept: <PRIOR_AUTO_ACCEPT>` as the last line of the `## Mission` block in the mission log (after `Estimated-Duration:`), so restore steps can parse it.
3. Update MT-4 step 5 and MT-3f step 5 to: read the `Prior-Auto-Accept:` line from the `## Mission` section of MISSION-LOG. If the value is `true`, skip removal and leave `autoAcceptEdits` set. Only remove `autoAcceptEdits` from `settings.local.json` if the value is `false` (or the field is absent from the log).

## Verification

### Manual tests
- Start a mission (MT-1), confirm `autoAcceptEdits` remains `true` in `settings.local.json` throughout.
- End the mission normally (MT-4 complete path) and confirm `autoAcceptEdits` is still `true` in `settings.local.json` afterward (not removed).
- End a mission via the halt path (MT-3f) and confirm the same.
- Edit a file outside a mission and confirm no diff popup appears.

### Automated tests
- Shell script: read `settings.local.json` before and after a simulated mission restore step; assert `autoAcceptEdits` is still `true` when it was pre-existing.
- No existing test harness in this project; a simple bash assertion would suffice.

## Changelog

### Review - 2026-03-25
- #1: Step 2 - replaced ambiguous "mission log front-matter or a comment" with a concrete storage location: `Prior-Auto-Accept:` field in the `## Mission` section of the mission log

### Review - 2026-03-25
- #2: Step 2 - specified exact `PRIOR_AUTO_ACCEPT` variable value (`true`/`false`), exact insertion position (after `Estimated-Duration:` in the `## Mission` block), and exact field format (`- Prior-Auto-Accept: <value>`)
- #3: Step 3 - specified how restore steps read `Prior-Auto-Accept:` from the mission log and the exact condition for skipping removal (`true`) vs. removing (`false` or absent)

## Progress
- Step 1: Confirmed `settings.local.json` line 27 has `"autoAcceptEdits": true` - key is already permanently present

## Implementation
Complete - 2026-03-25
