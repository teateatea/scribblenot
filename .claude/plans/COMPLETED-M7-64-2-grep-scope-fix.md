## Task

#64 - Add multi-file pattern search to Implementer prompt for repeated-pattern changes

## Context

Step 7 of the MT-3c Implementer prompt currently greps only `<PROJECT_ROOT>` when checking for additional occurrences of a changed pattern. However, Claude config files (SKILL.md files, hook scripts) live in `C:/Users/solar/.claude/` - a separate directory tree outside the project root. Mission 6 post-mortem identified a real failure where the Implementer updated SKILL.md in the project but missed an identical pattern in `pre-compact-mission-log.sh` (inside `.claude/`). Limiting grep to `PROJECT_ROOT` alone leaves the config tree unscanned, allowing sibling-file misses to recur.

## Approach

Edit step 7 in the MT-3c Implementer prompt (inside `SKILL.md`) to require two grep invocations: one over `<PROJECT_ROOT>` and one over `C:/Users/solar/.claude/`. Both use the same pattern and the same `--include="*"` flag. The Implementer must combine results from both trees when deciding whether additional locations need updating.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - line 330 (step 7 of the Implementer prompt)

## Reuse

No new utilities needed. The fix is a targeted text edit to the existing step 7 instruction.

## Steps

1. Read `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` in full to confirm line numbers before editing.

2. Replace the current step 7 text (single grep over PROJECT_ROOT) with a two-grep version covering both directories:

```diff
-> 7. Identify the primary text pattern or symbol changed in this sub-task (e.g. a field name, function name, or string literal). Run a case-sensitive grep across the full project root including the `hooks/` subdirectory: `grep -rn --include="*" "<PATTERN>" "<PROJECT_ROOT>"`. List every file containing the pattern. Update all additional matching locations that were not already changed in this sub-task. Record one of the following in your return value: `Grep: files found and updated: <list>`, `Grep: no additional matches`, or `Grep: N/A` (only if the sub-task change has no single identifiable repeated pattern).
+> 7. Identify the primary text pattern or symbol changed in this sub-task (e.g. a field name, function name, or string literal). Run a case-sensitive grep across BOTH of the following directory trees using the same pattern: (a) `grep -rn --include="*" "<PATTERN>" "<PROJECT_ROOT>"` and (b) `grep -rn --include="*" "<PATTERN>" "C:/Users/solar/.claude/"`. List every file containing the pattern from either search. Update all additional matching locations that were not already changed in this sub-task. Record one of the following in your return value: `Grep: files found and updated: <list>`, `Grep: no additional matches`, or `Grep: N/A` (only if the sub-task change has no single identifiable repeated pattern).
```

3. Verify the edit: re-read the modified section of SKILL.md and confirm step 7 now references both directories explicitly.

## Verification

### Manual tests

- Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and locate step 7 of the Implementer prompt. Confirm it now contains both `"<PROJECT_ROOT>"` and `"C:/Users/solar/.claude/"` as grep targets.

### Automated tests

- Doc check entries below cover the essential string assertions automatically.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | grep -rn --include="*" "<PATTERN>" "C:/Users/solar/.claude/"`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | BOTH of the following directory trees`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | Run a case-sensitive grep across the full project root including the`
