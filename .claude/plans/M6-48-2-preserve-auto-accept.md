## Task
#48 - Suppress diff windows globally when the auto-approve permission hook is active

## Context
`settings.local.json` already has `"autoAcceptEdits": true` permanently. The problem is that MT-4 step 5 and MT-3f step 5 unconditionally remove that key when a mission ends - reverting the permanent setting. The fix is to record whether `autoAcceptEdits` was already set before a mission starts, and skip removal at mission end if it was pre-existing.

## Approach
Record the pre-mission value of `autoAcceptEdits` as `Prior-Auto-Accept:` in the `## Mission` section of the mission log during MT-1 step 6. At mission end (MT-4 step 5 and MT-3f step 5), read that field and only remove the key if the prior value was `false` (or the field is absent).

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - MT-1 step 6 (line 96), MT-3f step 5 (line 441), MT-4 step 5 (line 524)
- `C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/settings.local.json` - already has `"autoAcceptEdits": true` (line 27); no change needed

## Reuse
- Existing `## Mission` block template in MT-1 step 5 (lines 53-85) - append new field after `Estimated-Duration:`
- Existing pattern of reading `Start-Time:` from `## Mission` in MT-3f step 2 and MT-4 step 2 - use same parsing approach for `Prior-Auto-Accept:`

## Steps

1. In `SKILL.md`, update MT-1 step 6. Before merging `autoAcceptEdits: true`, read the current `settings.local.json` and check whether the key exists and is `true`. Store as `PRIOR_AUTO_ACCEPT` (`true` if present and true, `false` otherwise). After writing `settings.local.json`, append `- Prior-Auto-Accept: <PRIOR_AUTO_ACCEPT>` as the last line of the `## Mission` section in MISSION_LOG_PATH (immediately after the `Estimated-Duration:` line; if `Estimated-Duration:` is absent, append after `Difficulty:`). The diff for step 6:

```
- 6. Suppress diff-view windows for the duration of this mission. Read `<PROJECT_ROOT>/.claude/settings.local.json` (create it as `{}` if absent). Merge `"autoAcceptEdits": true` into the top-level object and write it back. This prevents diff popups from interrupting the user during autonomous operation.
+ 6. Suppress diff-view windows for the duration of this mission. Read `<PROJECT_ROOT>/.claude/settings.local.json` (create it as `{}` if absent). Check whether `autoAcceptEdits` already exists and is `true`; store that result as `PRIOR_AUTO_ACCEPT` (`true` if the key is present and set to `true`, `false` otherwise). Merge `"autoAcceptEdits": true` into the top-level object and write it back. This prevents diff popups from interrupting the user during autonomous operation.
+    Then append `- Prior-Auto-Accept: <PRIOR_AUTO_ACCEPT>` as the last line of the `## Mission` section in `MISSION_LOG_PATH` (immediately after the `Estimated-Duration:` line; if `Estimated-Duration:` is absent, append after `Difficulty:`).
```

2. No change to the `## Mission` block template in MT-1 step 5 is needed. The `Prior-Auto-Accept:` field is appended to the live file in step 6 (after the template has already been written). Adding a literal placeholder like `(set in step 6)` to the template would cause that text to be written to disk and then produce a duplicate field when step 6 appends the real value. The template should reflect only what step 5 writes; step 6 handles the append.

3. In `SKILL.md`, update MT-3f step 5. Replace the unconditional removal with a conditional one:

```
- 5. Restore diff-view windows. Read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back. If the file would be empty after removal (`{}`), leave it as `{}`.
+ 5. Restore diff-view windows. Read `MISSION_LOG_PATH` and extract the `Prior-Auto-Accept:` value from the `## Mission` section. If the value is `true`, skip removal and leave `autoAcceptEdits` set. If the value is `false` or the field is absent, read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back. If the file would be empty after removal (`{}`), leave it as `{}`.
```

4. In `SKILL.md`, update MT-4 step 5. Apply the same guard:

```
- 5. Restore diff-view windows. Read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back.
+ 5. Restore diff-view windows. Read `MISSION_LOG_PATH` and extract the `Prior-Auto-Accept:` value from the `## Mission` section. If the value is `true`, skip removal and leave `autoAcceptEdits` set. If the value is `false` or the field is absent, read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back.
```

## Verification

### Manual tests
- Start a mission (MT-1); confirm `settings.local.json` still has `"autoAcceptEdits": true` and `MISSION_LOG_PATH` contains `- Prior-Auto-Accept: true` in the `## Mission` section.
- End the mission normally (MT-4 complete path); confirm `autoAcceptEdits` is still `true` in `settings.local.json` (not removed).
- End a mission via the halt path (MT-3f); confirm `autoAcceptEdits` is still `true` in `settings.local.json`.
- Edit a file outside a mission and confirm no diff popup appears.
- (Edge case) Manually remove `autoAcceptEdits` from `settings.local.json` before starting a mission; confirm MT-1 sets it to `true`, logs `Prior-Auto-Accept: false`, and MT-4/MT-3f removes it again at mission end.

### Automated tests
- Shell script: before and after a simulated MT-4 restore step with `Prior-Auto-Accept: true` in a mock log, assert `autoAcceptEdits` is still `true` in `settings.local.json`.
- Shell script: before and after a simulated MT-4 restore step with `Prior-Auto-Accept: false`, assert `autoAcceptEdits` is removed from `settings.local.json`.
- No existing test harness; bash assertions with `jq` are sufficient.

## Prefect-1 Report

### Nit

- **N1** (`M6-48-2-preserve-auto-accept.md:25`): The diff's `+` line for MT-1 step 6 omitted the fallback anchor `(if 'Estimated-Duration:' is absent, append after 'Difficulty:')` that was stated in the prose description on line 20. The diff is what gets written to SKILL.md, so the omission left the two authoritative sources inconsistent. Fixed by adding the fallback parenthetical to the diff's `+` line.

## Changelog

### Review - 2026-03-25
- #1 (blocking): Removed step 2 template diff that would have written literal `(set in step 6)` text to disk and created a duplicate `Prior-Auto-Accept:` field when step 6 appended the real value. Step 2 now explains that no template change is needed.
- #2 (nit): Added fallback anchor in step 1 append instruction: if `Estimated-Duration:` is absent, append after `Difficulty:` instead.

### Prefect-1 – 2026-03-25
- N1: Added fallback anchor `(if 'Estimated-Duration:' is absent, append after 'Difficulty:')` to the diff's `+` line in step 1 (plan line 25) to match the prose description on line 20.

## Progress
- Step 1: Updated MT-1 step 6 in SKILL.md to record PRIOR_AUTO_ACCEPT and append `Prior-Auto-Accept:` to MISSION_LOG_PATH
- Step 2: No template change needed (confirmed by plan)
- Step 3: Updated MT-3f step 5 in SKILL.md with conditional Prior-Auto-Accept guard
- Step 4: Updated MT-4 step 5 in SKILL.md with same conditional Prior-Auto-Accept guard
