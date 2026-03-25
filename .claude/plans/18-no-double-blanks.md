# Plan: 18-no-double-blanks

## Task
#18 - Prevent double blank lines in plan files

## Context
During previous missions, the Planner subagent introduced whitespace-only diffs by emitting double blank lines between sections in plan files. An audit found no current violations, but the root cause (missing explicit instructions) was confirmed in sub-task #18-2. The fix is preventive: add a one-sentence rule to the Planner prompt in `pathfinder-mission-team/SKILL.md` (MT-3c step 2), to the plan-writing step in `propose-plan/SKILL.md` (step 5), and fix the Changelog append template in `review-plan/SKILL.md` so successive entries never accumulate double blank lines between them.

## Approach
Edit the three skill files with the minimal wording needed to prevent double blank lines. No structural changes to the skills - only targeted sentence or template additions.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - MT-3c step 2 Planner subagent prompt (lines 186-201)
- `C:/Users/solar/.claude/skills/propose-plan/SKILL.md` - step 5 plan-writing instructions (lines 29-41)
- `C:/Users/solar/.claude/skills/review-plan/SKILL.md` - Changelog append template (lines 50-54)

## Reuse
No existing utilities to reuse - these are plain markdown edits.

## Steps

### Step 1: Add no-double-blank-lines rule to `propose-plan/SKILL.md` step 5

In step 5, after the instruction about verifying the slug doesn't already exist (line 31), add a formatting rule. The natural place is as a bullet after the existing slug-uniqueness check bullet, before the section list begins.

```diff
--- a/C:/Users/solar/.claude/skills/propose-plan/SKILL.md
+++ b/C:/Users/solar/.claude/skills/propose-plan/SKILL.md
@@ -31,6 +31,7 @@
    - Before writing, check whether the target filename already exists in `.claude/plans/` (use Glob). If it does, choose a different slug and check again until you find one that does not exist.
+   - **Formatting rule**: Use exactly one blank line between sections. Never write two or more consecutive blank lines anywhere in the plan file.
    - **Task** - the TASKS.md entry number and name this plan addresses (e.g. `#12 — Add highlight color support`). Omit if no match was found.
```

### Step 2: Add no-double-blank-lines rule to the Planner subagent prompt in `pathfinder-mission-team/SKILL.md`

The Planner subagent prompt is at MT-3c step 2 (lines 188-201). Add the formatting rule to the "Steps to follow" list, between step 4 (write the plan) and step 5 (return filename), so it applies at write time.

```diff
--- a/C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md
+++ b/C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md
@@ -197,6 +197,7 @@
 > 4. Write the plan to `<PROJECT_ROOT>/.claude/plans/<three-word-slug>.md` (verify slug does not already exist using Glob). Prefix with task number: e.g. `N-<slug>.md`.
+> 4a. **Formatting rule**: Use exactly one blank line between sections. Never write two or more consecutive blank lines anywhere in the plan file.
 > 5. Return ONLY the plan filename.
```

### Step 3: Fix the Changelog append template in `review-plan/SKILL.md`

The current template (lines 50-54) uses a fenced code block to show the Changelog entry format. The blank line inside the block between `### Review – <date>` and `- #N:` is fine, but successive appends by repeated reviewer passes can insert an extra blank line before the next `### Review` heading. Fix by adding an explicit note that there must be no blank line before a new `### Review` heading when appending to an existing Changelog section.

The current template block (Path A, lines 50-54):
```
        ```
        ### Review – <date>
        - #N: <one-line description of change made>
        ```
```

Change to add the no-double-blank note inline:

```diff
--- a/C:/Users/solar/.claude/skills/review-plan/SKILL.md
+++ b/C:/Users/solar/.claude/skills/review-plan/SKILL.md
@@ -50,6 +50,7 @@
       - Append a `## Changelog` section at the bottom of the plan file (create it if absent) with a new entry in this format:
         ```
         ### Review – <date>
         - #N: <one-line description of change made>
         ```
+        When appending to an existing `## Changelog` section, separate the new `### Review` heading from the previous entry with exactly one blank line — never two.
```

## Verification

### Manual tests
1. Run `/propose-plan` on any small task and open the resulting `.md` file. Confirm no two consecutive blank lines appear anywhere.
2. Run `/review-plan` on a plan that already has a `## Changelog` section. Confirm a second `### Review` entry is separated from the first by exactly one blank line.
3. Run a pathfinder mission on a single trivial task. Open the generated plan file and confirm no double blank lines.

### Automated tests
No automated test runner is present for skill `.md` files. A future option would be a shell script that runs `grep -Pzo '\n\n\n' .claude/plans/*.md` after each mission and asserts empty output.

### Doc checks
`C:/Users/solar/.claude/skills/propose-plan/SKILL.md | contains | Formatting rule: Use exactly one blank line between sections`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | 4a. **Formatting rule**`
`C:/Users/solar/.claude/skills/review-plan/SKILL.md | contains | exactly one blank line — never two`

## Changelog

### Review – 2026-03-25
- #1: Fixed malformed backtick in Doc check line 85 — the closing backtick after `Formatting rule` broke the `FILE | contains | STRING` pattern, causing verify-plan to match only "Formatting rule" instead of the full sentence.

## Progress
- Step 1: Added Formatting rule bullet to propose-plan/SKILL.md step 5 (after slug-uniqueness check)
- Step 2: Added step 4a Formatting rule to Planner subagent prompt in pathfinder-mission-team/SKILL.md
- Step 3: Added no-double-blank-lines note to Changelog append template in review-plan/SKILL.md

## Implementation
Complete – 2026-03-25
