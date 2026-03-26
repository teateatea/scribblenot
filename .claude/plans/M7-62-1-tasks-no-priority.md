# M7-62-1: Tasks No Priority

## Task

#62 - Omit (P:99) priority annotation from Tasks list in MISSION-LOG ## Mission section

## Context

The MT-1 initialization step writes the `- Tasks:` field in the MISSION-LOG `## Mission` section. The template placeholder instructs the LLM to include initial priorities alongside task IDs, resulting in output like `#64 (P:99), #66 (P:99), ...`. This annotation is redundant noise: the Priority column in `## Task Status` already tracks per-task priority, and the `(P:N)` suffix adds no information to the Tasks summary line. The fix is to change the template placeholder so the Tasks field emits plain comma-separated task IDs only.

## Approach

Edit the SKILL.md template placeholder text on the `- Tasks:` line so it instructs plain task IDs with no priority annotation. No logic changes are needed; the PRIORITY_MAP computation and Task Status table are unaffected.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` line 78

## Reuse

No existing utilities to reuse; this is a single-line doc change in the SKILL.md template.

## Steps

1. Edit line 78 of `SKILL.md` to remove the priority annotation from the Tasks placeholder:

```diff
- Tasks: <comma-separated list with initial priorities>
+ Tasks: <comma-separated task IDs, e.g. #64, #66, #65>
```

## Verification

### Manual tests

- After the change, invoke `/pathfinder-mission-team` with a BRIEF file or task list and confirm the generated `- Tasks:` line contains only plain task IDs (e.g. `#64, #66, #65`) with no `(P:N)` suffix.

### Automated tests

- No automated test runner covers SKILL.md template text; manual verification is the only option.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | comma-separated list with initial priorities`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | comma-separated task IDs, e.g. #64`

## Changelog

### Review - 2026-03-26
- #1: Changed plain code fence to ```diff fence on the Steps diff block for correct syntax highlighting.
