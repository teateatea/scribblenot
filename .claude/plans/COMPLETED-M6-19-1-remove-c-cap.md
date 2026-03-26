## Task

#19 - Remove the C:60 cap from add-task initial scoring; allow full 0-99 range

## Context

The add-task skill hard-codes a cap of 60 on the Clarity Confidence (C) score. This means even highly unambiguous tasks cannot score above 60 at intake. The design intent was to reserve high-confidence scores for a later review pass, but in practice it causes review passes to mechanically bump capped values upward rather than correcting toward accuracy. Removing the cap lets the initial score reflect genuine clarity and makes review passes meaningful.

## Approach

Edit `~/.claude/skills/add-task/SKILL.md` to remove all references to the C:60 cap. The score range becomes 1-99 (keeping 1 as the floor since 0 is reserved for Difficulty). Update the band descriptions to cover the full range: 1-30 Low/vague, 31-60 Mid/plannable, 61-99 High/clear. Remove "never above 60" enforcement language from both the inline categorization description and the agent prompt.

## Critical Files

- `C:/Users/solar/.claude/skills/add-task/SKILL.md` - lines 44-45 (D/C score definitions in Step 3) and lines 61-63 (agent prompt scoring rules)

## Reuse

No existing utilities to reuse. All changes are text edits within SKILL.md.

## Steps

1. Edit the C score definition in Step 3's categorization-pass description (line 45):

```
- **C score**: Clarity Confidence 1-60 (cap is intentional; 1-30 Low/vague, 31-60 Mid/plannable; never above 60 regardless of processing path)
+ **C score**: Clarity Confidence 1-99 (1-30 Low/vague, 31-60 Mid/plannable, 61-99 High/clear)
```

2. Edit the agent prompt's c_score line (lines 62-63 in the Agent prompt block):

```
- - "c_score": integer 1-60 (1-30 vague, 31-60 plannable; NEVER above 60)
+ - "c_score": integer 1-99 (1-30 vague, 31-60 plannable, 61-99 clear/unambiguous)
```

## Verification

### Manual tests

- Run `/add-task` with a task description that is completely unambiguous and self-contained (e.g. "Fix typo: change 'recieve' to 'receive' in the README"). Confirm the assigned C score can exceed 60 (e.g. C:75 or higher).
- Run `/add-task` with a vague task (e.g. "Make it faster"). Confirm the C score stays in the 1-30 range.
- Run `/add-task` with a moderately clear task. Confirm the C score lands in the 31-60 range.

### Automated tests

- No automated test suite exists for SKILL.md files. A scripted check could grep for the absence of "NEVER above 60" and "1-60" in the C score lines: `grep -n "NEVER above 60\|1-60" ~/.claude/skills/add-task/SKILL.md` should return no matches after the change.

### Doc checks

`C:/Users/solar/.claude/skills/add-task/SKILL.md | missing | never above 60 regardless of processing path`
`C:/Users/solar/.claude/skills/add-task/SKILL.md | missing | NEVER above 60`
`C:/Users/solar/.claude/skills/add-task/SKILL.md | contains | 61-99 High/clear`
`C:/Users/solar/.claude/skills/add-task/SKILL.md | contains | 61-99 clear/unambiguous`

## Progress

- Step 1: Updated C score definition on line 45 from "1-60 (cap is intentional...never above 60)" to "1-99 (1-30 Low/vague, 31-60 Mid/plannable, 61-99 High/clear)"
- Step 2: Updated agent prompt c_score on line 62 from "integer 1-60 (NEVER above 60)" to "integer 1-99 (61-99 clear/unambiguous)"

## Implementation
Complete - 2026-03-25
