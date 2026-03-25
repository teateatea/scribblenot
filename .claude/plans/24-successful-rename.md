## Task

#24 - Rename mission log to SUCCESSFUL-*.md when all tasks complete

## Context

When pathfinder-mission-team completes all tasks, the active MISSION-LOG file keeps its original name (e.g. `MISSION-LOG-6-auth-refactor.md`). There is no visual distinction between a completed mission log and one from a failed, abandoned, or still-in-progress run without opening the file. Adding a `SUCCESSFUL-` prefix at the end of MT-4 makes completed missions immediately distinguishable in the filesystem.

## Approach

Insert a new step between the existing MT-4 step 5 (restore diff-view windows) and step 6 (final output line). The new step runs `mv` to rename the log file to `SUCCESSFUL-<basename>`, then updates `MISSION_LOG_PATH` to the new path so the final output line references the correct file.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - lines 450-452 (MT-4 steps 5 and the output line)

## Reuse

- The same `mv` + `git add` fallback pattern used in MT-3d (lines 300-303) for COMPLETED- renames. Since `MISSION_LOG_PATH` lives under `<PROJECT_ROOT>/pathfinder/`, which may also be gitignored, use the same fallback: try `git -C "<PROJECT_ROOT>" mv` first; if it exits non-zero, run `mv` then `git -C "<PROJECT_ROOT>" add "<new path>"`.
- `MISSION_LOG_PATH` variable already holds the full path; `basename` of that path is extracted by stripping the directory prefix.

## Steps

1. Edit `SKILL.md` to insert a new step 5.5 between step 5 and the output line in MT-4:

```diff
 5. Restore diff-view windows. Read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back.

-Output: `Mission complete. See <MISSION_LOG_PATH> for full history.`
+5.5. Rename the mission log to `SUCCESSFUL-<basename>`. Derive `NEW_LOG_PATH` by prepending `SUCCESSFUL-` to the filename component of `MISSION_LOG_PATH` (the directory stays the same). Run:
+   - `git -C "<PROJECT_ROOT>" mv "pathfinder/<basename>" "pathfinder/SUCCESSFUL-<basename>"`
+   - If that exits non-zero (file not under version control), run `mv "<MISSION_LOG_PATH>" "<NEW_LOG_PATH>"` then `git -C "<PROJECT_ROOT>" add "pathfinder/SUCCESSFUL-<basename>"`.
+   - Update `MISSION_LOG_PATH = NEW_LOG_PATH`.
+   - Stage and commit: `git -C "<PROJECT_ROOT>" commit -m "Mission <MISSION_NUMBER> complete: rename log to SUCCESSFUL-<basename>"`
+
+Output: `Mission complete. See <MISSION_LOG_PATH> for full history.`
```

## Verification

### Manual tests

1. Run `/pathfinder-mission-team` on a single low-difficulty task through to completion.
2. After the mission ends, check `<PROJECT_ROOT>/pathfinder/` - the log file should be named `SUCCESSFUL-MISSION-LOG-<N>-<slug>.md`.
3. Confirm the final output line printed by MT-4 references the renamed path (i.e. `SUCCESSFUL-MISSION-LOG-...`).
4. Confirm `git log` shows a commit with message matching `Mission <N> complete: rename log to SUCCESSFUL-<basename>`.

### Automated tests

No automated test runner applies to SKILL.md edits. The verification above covers the observable behavior end-to-end.

## Changelog

### Review – 2026-03-25
- #1 (blocking): Fixed diff context line for step 5 — removed extra sentence "If the file would be empty after removal (`{}`), leave it as `{}`." that does not exist in the actual SKILL.md source at line 450.

## Progress
- Step 1: Inserted step 5.5 (SUCCESSFUL- rename + commit) between step 5 and the Output line in MT-4 of SKILL.md
