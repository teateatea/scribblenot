# Plan: M6-52-2 - Add re-read confirmation step to MT-3c Implementer subagent prompt

## Goal

After editing a SKILL.md file, hook script, or MISSION-PERMISSIONS.json, the Implementer subagent must immediately re-read the file in full, confirm structural validity, and record the result in its return value.

## Context

File: `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`

The MT-3c step 3 Implementer subagent prompt runs from line 276 to line 291. The numbered steps are:

1. Read implement-plan/SKILL.md
2. Read the plan file
3. Follow implement-plan Steps 3 and 4
4. Do NOT run git rebase / /verify / AskUserQuestion
5. Stage using explicit file names (no `git add -A`)
6. Commit with prescribed message format
7. Return exactly one of: IMPLEMENTED / FAILED

The return block currently begins immediately after step 6. A new step must be inserted between step 6 (commit) and the return block.

## Change

Insert a new step 7 into the Implementer subagent prompt, shifting the return block to follow it.

### Current text (lines 286-289)

```
> 6. Commit: `git -C "<PROJECT_ROOT>" commit -m "Implement task #<N> sub-task <SUB_ID>: <brief description>"`
> 7. Return exactly one of:
>    - `IMPLEMENTED: <one-line summary>`
>    - `FAILED: <reason>`
```

### Replacement text

```
> 6. Commit: `git -C "<PROJECT_ROOT>" commit -m "Implement task #<N> sub-task <SUB_ID>: <brief description>"`
> 7. If you edited a SKILL.md file, hook script, or MISSION-PERMISSIONS.json in this sub-task: re-read the edited file in full. Confirm it is structurally sound (no truncation, no broken markdown, no missing sections). Record `Re-read: Confirmed: <one-sentence summary of what was validated>` in your return value. If you did NOT edit any of these critical files, record `Re-read: N/A`.
> 8. Return exactly one of:
>    - `IMPLEMENTED: <one-line summary>` (include `Re-read: Confirmed: ...` or `Re-read: N/A` on the next line)
>    - `FAILED: <reason>`
```

## File to Edit

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`

Lines 286-289 (old step 6 + old step 7 return block).

## Exact Edit

Old string:
```
> 6. Commit: `git -C "<PROJECT_ROOT>" commit -m "Implement task #<N> sub-task <SUB_ID>: <brief description>"`
> 7. Return exactly one of:
>    - `IMPLEMENTED: <one-line summary>`
>    - `FAILED: <reason>`
```

New string:
```
> 6. Commit: `git -C "<PROJECT_ROOT>" commit -m "Implement task #<N> sub-task <SUB_ID>: <brief description>"`
> 7. If you edited a SKILL.md file, hook script, or MISSION-PERMISSIONS.json in this sub-task: re-read the edited file in full. Confirm it is structurally sound (no truncation, no broken markdown, no missing sections). Record `Re-read: Confirmed: <one-sentence summary of what was validated>` in your return value. If you did NOT edit any of these critical files, record `Re-read: N/A`.
> 8. Return exactly one of:
>    - `IMPLEMENTED: <one-line summary>` (include `Re-read: Confirmed: ...` or `Re-read: N/A` on the next line)
>    - `FAILED: <reason>`
```

## Verification

After editing:
1. Re-read the modified section of SKILL.md.
2. Confirm step numbers are sequential (1 through 8).
3. Confirm the return block is step 8.
4. Confirm no double blank lines were introduced.

## Status: Draft

## Changelog

### Review – 2026-03-25
- #1: Corrected line range in "Current text" heading from 286-290 to 286-289
- #2: Corrected line range in File to Edit section from 286-290 to 286-289
- #3: Aligned Replacement text block IMPLEMENTED note to match the more explicit phrasing in the Exact Edit New string

### Review – 2026-03-25 (Pass 2)
- No issues found; diff verified to apply cleanly at SKILL.md:286-289

### Review – 2026-03-25 (Pass 3)
- No issues found; plan confirmed correct, complete, and consistent
