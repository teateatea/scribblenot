## Task
#15 - Suppress diff-view windows during pathfinder-mission-team execution

## Context
File edits made by mission-team subagents trigger diff view windows that pop over the user's active window during autonomous operation. This creates a risk of accidental input if the user is typing elsewhere. The Claude Code setting `autoAcceptEdits: true` suppresses these diff popups. The setting should be written into the project-level `.claude/settings.local.json` at the start of a mission (MT-1) and removed at the end (MT-4 and MT-3f halt), so normal interactive use retains the diff windows the user wants to see.

## Approach
At MT-1 step 5 (after writing the mission log), merge `"autoAcceptEdits": true` into the project's `.claude/settings.local.json`. The file already exists in the scribblenot project with a `permissions` key; the implementer must read the existing JSON, add the new key at the top level, and write it back. At MT-4 and MT-3f halt (before logging completion/abandonment), remove the `autoAcceptEdits` key by reading the file, deleting the key, and writing it back. If the file does not exist at MT-1, create it with only `{"autoAcceptEdits": true}`.

The setting is written and removed in the SKILL.md instructions, not in any helper script, because the skill is the sole source of truth for the mission loop's behaviour.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - lines 37-79 (MT-1 section, specifically step 5), lines 333-348 (MT-3f halt), lines 354-372 (MT-4 mission complete)

## Reuse
- The existing SKILL.md pattern of writing to `MISSION_LOG_PATH` via Bash is the model for writing JSON with a Bash subagent step. The implementer should use `python -c` or `node -e` inline one-liners to read, merge, and write the JSON (avoids dependency on jq which may not be present on Windows). Note: use `python` not `python3` on Windows (see refresh-mirror skill precedent).

## Steps

1. In SKILL.md, after MT-1 step 5 (the `Record MISSION_LOG_PATH = ...` line, line ~79), insert a new step 6:

```
-Record `MISSION_LOG_PATH` = the full path to this file (i.e. `<PROJECT_ROOT>/pathfinder/MISSION-LOG-<N>-<MISSION_SLUG>.md`).
+Record `MISSION_LOG_PATH` = the full path to this file (i.e. `<PROJECT_ROOT>/pathfinder/MISSION-LOG-<N>-<MISSION_SLUG>.md`).
+
+6. Suppress diff-view windows for the duration of this mission. Read `<PROJECT_ROOT>/.claude/settings.local.json` (create it as `{}` if absent). Merge `"autoAcceptEdits": true` into the top-level object and write it back. This prevents diff popups from interrupting the user during autonomous operation.
```

2. In SKILL.md, in the MT-3f halt block, before step 5 ("Stop."), insert a restore step:

```
+5. Restore diff-view windows. Read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back. If the file would be empty after removal (`{}`), leave it as `{}`.
+
-5. Stop.
+6. Stop.
```

3. In SKILL.md, in the MT-4 mission complete block, after step 4 (the markdown block with the Mission Complete template) and before the `Output:` line, insert a restore step:

```
+5. Restore diff-view windows. Read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back.
+
 Output: `Mission complete. See <MISSION_LOG_PATH> for full history.`
```

## Verification

### Manual tests
1. Start a pathfinder-mission-team run on any task. Confirm that no diff popup windows appear when the mission team edits files.
2. After the mission completes (MT-4) or is abandoned (MT-3f), confirm that editing a file manually in the Claude Code session again produces a diff popup as normal.
3. Open `.claude/settings.local.json` mid-mission and confirm `"autoAcceptEdits": true` is present; open it after mission completion and confirm the key is absent.

### Automated tests
- No automated test runner is present for SKILL.md changes (doc-only). The verification is observational.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | autoAcceptEdits`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Restore diff-view windows`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Suppress diff-view windows`

## Changelog

### Review - 2026-03-25
- #1: Changed `python3 -c` to `python -c` in the Reuse section to match the Windows-specific convention established in the refresh-mirror skill (use `python` not `python3` on Windows).

### Prefect Pass 1 - 2026-03-25
- Blocking: Step 2 diff block was inconsistent - showed `5. Stop.` as unchanged context while prose said to renumber it to `6. Stop.`. Fixed diff to show `-5. Stop.` / `+6. Stop.` pair and removed the now-redundant renumbering prose (15-suppress-diff-windows.md:34-37).

## Progress
- Step 1: Inserted MT-1 step 6 (Suppress diff-view windows) after the Record MISSION_LOG_PATH line in SKILL.md
- Step 2: Inserted MT-3f restore step 5 and renumbered Stop to step 6 in SKILL.md
- Step 3: Inserted MT-4 restore step 5 before Output line in SKILL.md

## Implementation
Complete - 2026-03-25
