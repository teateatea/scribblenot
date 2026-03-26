## Task

#57 - Fix M6 Start-Time recorded ~4 hours ahead of actual local time

## Context

Sub-task 57.2 replaced `TZ=America/Toronto date` with plain `date` in two files: `pathfinder-mission-team/SKILL.md` (10 occurrences) and `.claude/hooks/pre-compact-mission-log.sh` (1 occurrence). This sub-task audits whether any other files were missed. A full grep of `C:/Users/solar/.claude/` (excluding file-history cache) and `C:/Users/solar/Documents/Claude Projects/scribblenot/` reveals 2 remaining live occurrences in `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` at lines 20 and 231. All other matches in the scribblenot repo are read-only log/history artifacts (MISSION-LOG files, SUCCESSFUL-MISSION-LOG files) that record past events and must not be changed.

## Approach

Replace both `TZ=America/Toronto date` occurrences in `pathfinder-premission/SKILL.md` with plain `date`, matching the same substitution pattern used in sub-task 57.2. No other live files require changes.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` - lines 20 and 231 (two occurrences to replace)

## Reuse

Same replacement pattern as sub-task 57.2: substitute `TZ=America/Toronto date` with `date`, preserving the rest of the timestamp format string unchanged.

## Steps

1. Read `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` to confirm current content at lines 20 and 231.

2. Replace both occurrences using Edit with `replace_all: true`:

```
- TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"
+ date +"%Y-%m-%dT%H:%M:%S"
```

3. Grep `pathfinder-premission/SKILL.md` for `TZ=America/Toronto date` and confirm zero matches remain.

4. Grep all of `C:/Users/solar/.claude/skills/` for `TZ=America/Toronto date` and confirm zero matches remain across all skill files.

## Verification

### Manual tests

- None required; this is a text substitution in a skill instruction file with no runtime component to exercise interactively.

### Automated tests

- `grep -rn "TZ=America/Toronto date" C:/Users/solar/.claude/skills/` must return no output (exit code 1 / zero matches).
- `grep -n "date +\"%Y-%m-%dT%H:%M:%S\"" C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` must return lines 20 and 231 with the plain `date` form, confirming the format string is intact.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | missing | TZ=America/Toronto date`
