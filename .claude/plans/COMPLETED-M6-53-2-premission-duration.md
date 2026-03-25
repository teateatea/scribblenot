# Plan: M6-53-2 - Add Estimated Duration Display After D-Check in pathfinder-premission/SKILL.md

Status: Draft

## Goal

After the difficulty sum check (step 4.5) in PM-1 resolves - regardless of which branch the user took - display a single informational note to the user: "Estimated duration: ~X min (D x 0.43)" where X = round(total_D * 0.43).

## Context

File: `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md`

The difficulty sum check lives in step 4.5, lines 25-29. All three branches (sum > 280, sum > 140, sum <= 140) eventually continue to step 5. The insertion point is between step 4.5 and step 5 - after the sum check concludes and before the D/C threshold check begins.

The formula matches the existing threshold comments in the skill: the 280 threshold is annotated as "~2 hours" and the 140 threshold as "~1 hour". 280 * 0.43 = 120.4 min (~2 hours); 140 * 0.43 = 60.2 min (~1 hour). The formula is internally consistent with the existing thresholds.

## Implementation

### Change 1: Insert duration display paragraph at the end of step 4.5

In `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md`, locate the end of step 4.5. The current last line of step 4.5 is:

```
   - If `sum <= 140`: no action needed, proceed directly to step 5.
```

Insert the following paragraph immediately after that line, before the blank line that separates step 4.5 from step 5:

```
   After whichever branch above resolves (whether the user confirmed, adjusted, or skipped), output this note (no question, no AskUserQuestion): **"Estimated duration: ~X min (total_D x 0.43)"** where X = `round(total_D * 0.43)` and `total_D` is the D-score sum of the final confirmed task list at this point.
```

This is a single output statement - not a question, not an interactive step.

## Minimal Diff

Old text (lines 28-31):
```
   - Else if `sum > 140`: Use AskUserQuestion (single-select) to display: "Note: Total mission difficulty is <sum>, which exceeds 140 (~1 hour estimated). The main instance context may be overtaxed before all tasks complete. Do you want to continue with this task list?" with options: "Yes, continue" and "No, I will reduce the task list". If the user selects "No": if ARGUMENTS were pre-specified (task numbers passed in), return to step 4 and prompt the user to remove tasks manually from the list; otherwise return to step 3 to let them change the task selection. If "Yes", continue.
   - If `sum <= 140`: no action needed, proceed directly to step 5.

5. **D/C threshold check.**
```

New text:
```
   - Else if `sum > 140`: Use AskUserQuestion (single-select) to display: "Note: Total mission difficulty is <sum>, which exceeds 140 (~1 hour estimated). The main instance context may be overtaxed before all tasks complete. Do you want to continue with this task list?" with options: "Yes, continue" and "No, I will reduce the task list". If the user selects "No": if ARGUMENTS were pre-specified (task numbers passed in), return to step 4 and prompt the user to remove tasks manually from the list; otherwise return to step 3 to let them change the task selection. If "Yes", continue.
   - If `sum <= 140`: no action needed, proceed directly to step 5.

   After whichever branch above resolves (whether the user confirmed, adjusted, or skipped), output this note (no question, no AskUserQuestion): **"Estimated duration: ~X min (total_D x 0.43)"** where X = `round(total_D * 0.43)` and `total_D` is the D-score sum of the final confirmed task list at this point.

5. **D/C threshold check.**
```

## Notes

- No new questions, no new AskUserQuestion calls - this is a display-only change.
- Uses the final confirmed task list's D sum, so if the user reduced tasks in the >280 or >140 branch, the estimate reflects the updated list.
- The blank line before "5." is preserved to maintain section spacing.
- No consecutive blank lines introduced.
- Formula validation (per MISSION-6-BRIEF constraint for task #53-2): M5 actual data shows Duration=168 min, total D=390, ratio=168/390=0.431, confirming the 0.43 constant. No correction needed.

## Changelog

### Review - 2026-03-25
- #1: Added formula validation note citing M5 data (D=390, Duration=168 min, ratio=0.431) to satisfy the MISSION-6-BRIEF constraint requiring validation before baking in the 0.43 constant.

## Prefect-1 Report

### Issues Found

**Nit - line number label in Minimal Diff header**
The "Old text" block originally said "lines 27-29" but the block actually covers lines 28-31 of the target file (line 28: `sum > 140` bullet; line 29: `sum <= 140` bullet; line 30: blank line; line 31: `5. **D/C threshold check.**`). The diff itself applies cleanly - this was a label-only error.

### Verdict

Nit only. Fix applied directly. Diff verified to apply cleanly against actual pathfinder-premission/SKILL.md content.

## Prefect-2 Report

### Issues Found

**Nit - "After the branch above resolves" is slightly ambiguous**
The inserted paragraph begins "After the branch above resolves (whether the user confirmed or adjusted)". "The branch above" could be read as referring only to the immediately preceding `sum <= 140` bullet (which requires no user action) rather than whichever of the three branches executed. Suggested wording: "After whichever branch above resolves (whether the user confirmed, adjusted, or skipped)".

**Nit - Display string uses "D" ambiguously**
The output note reads `**"Estimated duration: ~X min (D x 0.43)"**`. Within the skill, "D" consistently refers to an individual task's D score. The display string's "D" means total D sum, which could confuse a reader. The inline explanation (`where X = \`round(total_D * 0.43)\``) clarifies it, but the display string itself is ambiguous. Suggested wording: `**"Estimated duration: ~X min (total_D x 0.43)"**`.

### Verdict

Nits only. Diff applies cleanly. Formula validated. Implementation intent is sound.

## Changelog

### Prefect-1 - 2026-03-25
- #1: Corrected "Old text (lines 27-29)" label to "Old text (lines 28-31)" to match actual line numbers in target file.

### Review #2 - 2026-03-25
- #1: Applied Prefect-2 nit: changed "After the branch above resolves (whether the user confirmed or adjusted)" to "After whichever branch above resolves (whether the user confirmed, adjusted, or skipped)" in both the Implementation and Minimal Diff sections.
- #2: Applied Prefect-2 nit: changed `(D x 0.43)` to `(total_D x 0.43)` in the display string in both the Implementation and Minimal Diff sections.

## Progress
- Step 1 (Change 1): Inserted duration display paragraph after `sum <= 140` line in pathfinder-premission/SKILL.md step 4.5

## Implementation
Complete - 2026-03-25
