## Task

#42 - Rename PROJECT-FOUNDATION to MISSION-#-BRIEF and add task priority order to it in both pathfinder skills

## Context

`pathfinder-premission/SKILL.md` hardcodes the output filename as `PROJECT-FOUNDATION.md` in PM-4 and PM-6. The mission number (slug) is already derived in PM-3, so the brief filename can be made mission-specific. Additionally, the approved task priority order collected in PM-1 is not written into the foundation file, leaving the mission team without a canonical record of execution order.

This sub-task covers only `pathfinder-premission/SKILL.md`. A companion sub-task handles `pathfinder-mission-team/SKILL.md`.

## Approach

In `pathfinder-premission/SKILL.md`:

1. After PM-3 produces the mission slug, derive `MISSION_NUMBER` as a plain integer counter (the next integer after the highest existing `MISSION-N-*` file in `pathfinder/`). Assign it at the end of PM-3 so PM-4 and later steps can use it.
2. Replace every hard-coded `PROJECT-FOUNDATION.md` reference with `MISSION-<MISSION_NUMBER>-BRIEF.md`.
3. Extend the Foundation Author subagent prompt (PM-4) to include the confirmed task priority order as a new "Task Priority Order" section in the output document.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` — lines 98-170 (PM-3 and PM-4), line 214 (PM-6 summary), line 219 (PM-6 file list)

## Reuse

- The mission slug derivation already exists in PM-3 (line 98). `MISSION_NUMBER` is a small extension of that logic.
- The Foundation Author subagent prompt template (lines 137-168) is extended in-place; no new subagent type is needed.

## Steps

1. At the end of PM-3, after writing `MISSION-PERMISSIONS.json`, add a step that computes `MISSION_NUMBER`:

```
- After writing MISSION-PERMISSIONS.json, determine MISSION_NUMBER: glob `<PROJECT_ROOT>/pathfinder/MISSION-*-BRIEF.md`; if none exist, MISSION_NUMBER = 1; otherwise MISSION_NUMBER = (highest N found) + 1. Store as `MISSION_NUMBER` for use in PM-4 through PM-6.
```

2. In the PM-3 section heading comment and slug line, no filename change is needed (slug stays as-is).

3. In PM-4, replace the output filename instruction and the `Write` line:

```diff
-> Write a PROJECT-FOUNDATION.md document with exactly these five sections in this order:
+> Write a MISSION-<MISSION_NUMBER>-BRIEF.md document with exactly these six sections in this order:
```

4. Add a new "Task Priority Order" section to the document template inside the Foundation Author prompt, inserted between "## Requirements" and "## Explicit Non-Goals":

```diff
 > ## Requirements
 > - <Requirement 1>
 > - <Requirement 2>
 >
+> ## Task Priority Order
+> <Ordered list of task numbers and titles, highest priority first, exactly as confirmed by the user in PM-1. This is the authoritative execution order for the mission team.>
+> - #N - <title>
+> - #M - <title>
+>
 > ## Explicit Non-Goals
```

5. Update the Foundation Author prompt preamble to pass the confirmed task list in priority order:

```diff
 > Additionally, the following pre-mission clarification notes were captured from the user for ambiguous tasks. Treat each note as a direct constraint or requirement for its task:
 > <for each task that has a Pre-mission note, insert a line: "Task #<N>: <Pre-mission note text>">
 > (Omit this section if no tasks had clarification notes.)
+>
+> The confirmed task priority order (highest priority first) is:
+> <for each task in the confirmed list, in order: "- #<N>: <title>">
+> The foundation document must record this order verbatim in the "Task Priority Order" section.
```

6. Update the `Write` call at the end of PM-4:

```diff
-Write the approved version to `<PROJECT_ROOT>/pathfinder/PROJECT-FOUNDATION.md`.
+Write the approved version to `<PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md`.
```

7. Update the PM-6 pre-flight summary template:

```diff
-Foundation: PROJECT-FOUNDATION.md (5 sections: Goals, Requirements, Explicit Non-Goals, Constraints, Test Criteria)
+Foundation: MISSION-<MISSION_NUMBER>-BRIEF.md (6 sections: Goals, Requirements, Task Priority Order, Explicit Non-Goals, Constraints, Test Criteria)
```

8. Update the PM-6 file list:

```diff
-- <PROJECT_ROOT>/pathfinder/PROJECT-FOUNDATION.md
+- <PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md
```

9. Update the PM-4 section heading to reflect the new output filename:

```diff
-### PM-4: Produce PROJECT-FOUNDATION.md
+### PM-4: Produce MISSION-<MISSION_NUMBER>-BRIEF.md
```

10. Update the skill `description` front-matter line. Use `MISSION-N-BRIEF.md` (not `<MISSION_NUMBER>`) because this is a static metadata field - `N` conveys "per-mission numbered file" without implying runtime substitution:

```diff
-description: Guides the user through pre-mission setup for pathfinder-mission-team. Produces pathfinder/MISSION-PERMISSIONS.json, pathfinder/PROJECT-FOUNDATION.md, and Project Tests. Ends by confirming the user is ready to go dark. Trigger on "/pathfinder-premission".
+description: Guides the user through pre-mission setup for pathfinder-mission-team. Produces pathfinder/MISSION-PERMISSIONS.json, pathfinder/MISSION-N-BRIEF.md, and Project Tests. Ends by confirming the user is ready to go dark. Trigger on "/pathfinder-premission".
```

## Verification

### Manual tests

- Run `/pathfinder-premission` on a project with no existing `MISSION-*-BRIEF.md` files; confirm the file created is `MISSION-1-BRIEF.md` and contains a "Task Priority Order" section listing tasks in the order confirmed at PM-1.
- Run a second premission on the same project; confirm the new file is `MISSION-2-BRIEF.md`.
- Open `MISSION-1-BRIEF.md` and verify the task order matches what was confirmed in PM-1.

### Automated tests

- Doc check: after a run, assert the brief file contains the string `## Task Priority Order`.
- Unit test (bash): glob `pathfinder/MISSION-*-BRIEF.md` and assert the count increments correctly across two sequential premission runs.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | missing | PROJECT-FOUNDATION.md`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | MISSION-<MISSION_NUMBER>-BRIEF.md`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | Task Priority Order`

## Changelog

### Review - 2026-03-25
- #1 (minor): Added step 9 - diff to update PM-4 section heading from "Produce PROJECT-FOUNDATION.md" to "Produce MISSION-<MISSION_NUMBER>-BRIEF.md" (was missing from original plan)
- #2 (nit): Updated step 10 (was step 9) - clarified that front-matter description uses `MISSION-N-BRIEF.md` (static placeholder) rather than `<MISSION_NUMBER>` (runtime variable) to avoid misleading implementers; added explicit diff block

### Review - 2026-03-25
- #3 (minor): Approach step 1 said "zero-padded counter" but step 1's proposed text uses plain integers and existing mission files (MISSION-LOG-4, MISSION-LOG-5) use no padding; changed "zero-padded" to "plain integer" to match actual behavior

### Review - 2026-03-25
- N1 (nit): Step 5 diff context line expanded to full source text to match `pathfinder-premission/SKILL.md:141` exactly

## Prefect-1 Report

### Nit

- **N1** (`M6-42-1-premission-brief-rename.md:63`): Step 5 diff used a truncated context line (`> Additionally, the following pre-mission clarification notes were captured...`) that does not match the actual source text in `pathfinder-premission/SKILL.md:141`. Replaced with the full source line so the diff context is unambiguous for implementers.

## Prefect-2 Report

### Blocking

- **B1** (`M6-42-1-premission-brief-rename.md:41-43`, `pathfinder-premission/SKILL.md:145`): Step 3 diff `-` line is missing the `> ` blockquote prefix. The actual source reads `> Write a PROJECT-FOUNDATION.md document with exactly these five sections in this order:` (with leading `> `). The diff as written will not apply cleanly because the `-` context does not match the file.

- **B2** (`M6-42-1-premission-brief-rename.md:47-57`, `pathfinder-premission/SKILL.md:153-157`): Step 4 diff context lines (` ## Requirements`, ` - <Requirement 1>`, ` - <Requirement 2>`, ` ## Explicit Non-Goals`) and all new `+` lines are missing the `> ` blockquote prefix present in the actual source. The source lines 153-157 all begin with `> `. The diff will not apply cleanly, and if forced, would insert the new `## Task Priority Order` section without its required `> ` prefix, breaking the subagent prompt blockquote formatting.

### Review - 2026-03-25
- B1: Step 3 diff `-` line added missing `> ` blockquote prefix to match `pathfinder-premission/SKILL.md:145` exactly
- B2: Step 4 diff context and `+` lines added missing `> ` blockquote prefix (including blank `>` line) to match `pathfinder-premission/SKILL.md:153-157` exactly

### Review - 2026-03-25
- #1 (minor): Step 5 diff context expanded to include source lines 142-143 (`> <for each task...>` and `> (Omit this section...)`) so the insertion point falls after the full clarification-notes block rather than splitting it mid-block

## Progress

- Step 1: Added MISSION_NUMBER computation at end of PM-3 (after MISSION-PERMISSIONS.json write)
- Step 2: No change needed (slug stays as-is)
- Step 3: Updated Foundation Author prompt preamble - "five sections" to "six sections", PROJECT-FOUNDATION.md to MISSION-<MISSION_NUMBER>-BRIEF.md
- Step 4: Added Task Priority Order section to document template between Requirements and Explicit Non-Goals
- Step 5: Added confirmed task priority order block to Foundation Author subagent prompt (after clarification notes block)
- Step 6: Updated Write call at end of PM-4 to use MISSION-<MISSION_NUMBER>-BRIEF.md
- Step 7: Updated PM-6 Foundation line to MISSION-<MISSION_NUMBER>-BRIEF.md with 6 sections including Task Priority Order
- Step 8: Updated PM-6 file list entry from PROJECT-FOUNDATION.md to MISSION-<MISSION_NUMBER>-BRIEF.md
- Step 9: Updated PM-4 section heading to "Produce MISSION-<MISSION_NUMBER>-BRIEF.md"
- Step 10: Updated front-matter description to use MISSION-N-BRIEF.md (static placeholder)

## Implementation
Complete - 2026-03-25

## Prefect-3 Report

### Nit

- **N1** (`M6-42-1-premission-brief-rename.md:66`, `pathfinder-premission/SKILL.md:143-144`): Step 5 diff's first `+` line is `+>` (a blank blockquote line), but source line 144 (`>`) already provides a blank separator before the "Write a PROJECT-FOUNDATION.md..." line. Applying this diff as written inserts the new text between lines 143 and 144, resulting in two consecutive blank `>` lines (the newly-inserted one plus the pre-existing line 144) before the "Write" instruction. The `+>` line should be removed from the diff; the existing blank line 144 already serves as the paragraph separator.
