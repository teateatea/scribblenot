## Task

#56-2 - Track per-mission command hit counts in DEFAULT-PERMISSIONS

## Context

DEFAULT-PERMISSIONS.json has `mission_use_count` fields on each entry (added by sub-task 1), but nothing increments them. When a mission completes, the Commander has no record of which Bash-type approved_actions were actually invoked during the run. This plan wires up that tracking: the Commander maintains a USED_COMMANDS set across all sub-tasks, the Implementer reports which Bash patterns it invoked, and at MT-4 the Commander increments `mission_use_count` for each used Bash entry in DEFAULT-PERMISSIONS.json.

## Approach

Add a `USED_COMMANDS` set to the Commander's mission state (initialized to the empty set in the MT-3 state block at line 143). After each Implementer subagent returns `IMPLEMENTED`, the Commander parses the new `Bash-used:` line from the return value and adds those patterns to USED_COMMANDS. At MT-4 (after `## Mission Complete` is appended), the Commander reads DEFAULT-PERMISSIONS.json, increments `mission_use_count` for each `"type": "Bash"` entry whose `pattern` appears in USED_COMMANDS, and writes the file back. The Bash pattern match is exact-string (no glob expansion needed - match the `pattern` field value against the reported strings verbatim).

Tracking is binary per mission: a pattern is either in USED_COMMANDS or not. Multiple invocations of the same pattern still only increment the counter by 1.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` (lines 143, 300-319, 469-565) - MT-3 state initialization, Implementer prompt, MT-4 steps

## Reuse

- Existing `python3 -c` Bash pattern already approved in MISSION-PERMISSIONS.json for JSON read/write (line 32)
- Existing DEFAULT-PERMISSIONS.json read/write pattern established by sub-task 1

## Steps

1. **Add USED_COMMANDS to MT-3 state initialization (line 143 of SKILL.md)**

   In the paragraph beginning "Maintain a TASK_QUEUE...", add USED_COMMANDS to the state list after PRIOR_ATTEMPT_MAP:

   ```
   - a PRIOR_ATTEMPT_MAP (task -> list of prior-attempt records, ... each record is added on the MT-3d failure branch ...)
   + , and a USED_COMMANDS set (set of Bash pattern strings that were invoked during this mission, initialized to the empty set; populated from Implementer return values during MT-3c)
   ```

   The sentence should read: `...and a PRIOR_ATTEMPT_MAP (task -> list of prior-attempt records, initialized to [] per task; each record is added on the MT-3d failure branch and contains the sub-tasks that ran and the project-test criteria that failed), and a USED_COMMANDS set (set of Bash pattern strings that were invoked during this mission, initialized to the empty set; populated from Implementer return values during MT-3c).`

2. **Extend the Implementer prompt (MT-3c step 3, lines 300-319 of SKILL.md) to report Bash patterns used**

   After step 8 ("Re-read:") in the Implementer prompt, insert a new step 8.5 (the existing `9. Return exactly one of:` retains its number as step 9 - no renumbering needed):

   ```
   - old (step 9 begins with): > 9. Return exactly one of:
   + new (insert before step 9):
   > 8.5. Identify every distinct Bash pattern from `<PROJECT_ROOT>/pathfinder/MISSION-PERMISSIONS.json` `approved_actions` whose `type` is `"Bash"` and whose `pattern` you actually invoked at least once during this sub-task. Report them as a comma-separated list. If no Bash commands matching an approved pattern were run, report "none".
   ```

   And extend the return format for `IMPLEMENTED` to include a `Bash-used:` line:

   ```
   - old:
   >    - `IMPLEMENTED: <one-line summary>` (include `Grep: <result>` and `Re-read: Confirmed: ...` or `Re-read: N/A` on separate lines)
   + new:
   >    - `IMPLEMENTED: <one-line summary>` (include `Grep: <result>`, `Re-read: Confirmed: ...` or `Re-read: N/A`, and `Bash-used: <comma-separated list of Bash patterns invoked, or "none">` on separate lines)
   ```

3. **Add Commander logic to collect USED_COMMANDS after each Implementer return (MT-3c step 3)**

   Insert as a new standalone paragraph immediately after the Implementer subagent block (between line 319 and line 321 of SKILL.md, between `> Do NOT use AskUserQuestion.` and `**4. Verify TDD tests pass**`):

   ```
   + After the Implementer returns IMPLEMENTED, parse the `Bash-used:` line from its output. For each pattern listed (skip "none"), add the pattern string to USED_COMMANDS.
   ```

4. **Add DEFAULT-PERMISSIONS update step at MT-4 (after step 4b, before step 5)**

   Insert new step 4c in the MT-4 section:

   ```
   4c. Update command hit counts. If `<PROJECT_ROOT>/pathfinder/DEFAULT-PERMISSIONS.json` exists and USED_COMMANDS is non-empty:
       - Read `<PROJECT_ROOT>/pathfinder/DEFAULT-PERMISSIONS.json`.
       - For each entry in `approved_actions` where `"type"` is `"Bash"` and the entry's `"pattern"` value is present in USED_COMMANDS: increment its `mission_use_count` by 1.
       - Write the updated JSON back to `<PROJECT_ROOT>/pathfinder/DEFAULT-PERMISSIONS.json` (preserve formatting; use python3 -c for the read/increment/write).
       - Stage and commit: `git -C "<PROJECT_ROOT>" add "pathfinder/DEFAULT-PERMISSIONS.json"` then `git -C "<PROJECT_ROOT>" commit -m "Mission <MISSION_NUMBER> complete: update DEFAULT-PERMISSIONS command use counts"`.
       - If DEFAULT-PERMISSIONS.json does not exist or USED_COMMANDS is empty, skip this step entirely.
   ```

5. **Add `Bash-used:` field to the sub-task log template (step 5 of MT-3c, lines 330-341 of SKILL.md)**

   The sub-task log template mirrors the Implementer return fields (Shim-removal, Grep, Re-read each have their own row). Add `Bash-used:` after `Re-read:` and before `Agent:`:

   ```
   - Bash-used: <"N/A" if the sub-task type is not Bash-invoking | comma-separated list of Bash patterns invoked | "none" if no approved Bash patterns were run | "Absent" if the Implementer did not report this field>
   ```

   This ensures the patterns are persisted in the mission log for auditability, consistent with how Grep: and Re-read: are recorded.

   Note: `Bash-used:` is intentionally excluded from the MT-3d log enforcement gate (SKILL.md lines 348-379). The enforcement gate hard-blocks on `Agent` and `Re-read` absence and soft-blocks on `Status`/`Implementation`/`Timestamp` absence because those fields are critical for mission integrity. `Bash-used:` is an audit field whose absence should not block task completion - if the Implementer omits it, the value "Absent" in the log template is sufficient to flag the gap without a hard block. No change to the enforcement gate is required.

## Verification

### Manual tests

1. Run a mission with a task that executes at least one known Bash pattern (e.g. `git *`). After mission completes, open `pathfinder/DEFAULT-PERMISSIONS.json` and confirm `mission_use_count` for that `git *` entry is incremented by 1 compared to its pre-mission value.
2. Run a second mission that also uses `git *`. Confirm the count increments to 2 (cumulative across missions).
3. Confirm that non-Bash entries (Read, Write, Edit) and Bash entries whose patterns were not invoked have unchanged `mission_use_count`.
4. Confirm the sub-task log entry for an Implementing sub-task includes a `Bash-used:` row listing the patterns invoked.

### Automated tests

No automated test runner covers SKILL.md logic directly. The verification is observational (JSON file inspection post-mission). A future integration test could mock an Implementer subagent and assert that DEFAULT-PERMISSIONS.json is updated correctly, but none exists today.

## Prefect-1 Report

### Changes Made

- **[minor] Approach section, line 11**: "initialized at MT-1" was incorrect - USED_COMMANDS is added to the MT-3 state block (line 143 of SKILL.md), not MT-1. MT-1 only initializes the mission log file. All other Commander state variables (TASK_QUEUE, PRIORITY_MAP, etc.) live in the MT-3 block. Fixed to say "initialized to the empty set in the MT-3 state block at line 143".

## Prefect-2 Report

### Issues Found

**[minor] Step 3 / SKILL.md lines 319-321 - "prose following the Implementer prompt block" does not exist**

The plan says to add the Commander USED_COMMANDS logic "in the same paragraph or as a new sentence immediately after the existing Implementer subagent block, before step 4 'Verify TDD tests pass'." However, in the actual SKILL.md there is no prose between the end of the Implementer block (line 319, `> Do NOT use AskUserQuestion.`) and step 4 (line 321, `**4. Verify TDD tests pass**`). There is a single blank line separating them with no existing paragraph to insert into. The implementer must create new standalone text rather than appending to existing prose. The step text should be clarified: replace "In the same paragraph or as a new sentence immediately after the existing Implementer subagent block" with "Insert as a new standalone paragraph immediately after the Implementer subagent block (between line 319 and line 321 of SKILL.md), before step 4".

**[minor] Step 5 / SKILL.md lines 329-341 - `Bash-used:` field not added to MT-3d enforcement gate**

The plan adds `Bash-used:` to the sub-task log template (Step 5) but does not add it to the MT-3d log enforcement gate (SKILL.md lines 348-379). The enforcement gate currently checks for `Agent` (hard block), `Re-read` (hard block), and `Status`/`Implementation`/`Timestamp` (soft hard block). If `Bash-used:` is expected to always be present in the log (with "N/A", "none", or "Absent" as valid values per the template definition), then its absence should be enforced. If intentionally not enforced, the plan should state this explicitly. Either add a Step 6 to add `Bash-used:` to the enforcement gate, or add a note to Step 5 explaining why it is excluded from enforcement.

**[nit] Step 2 / SKILL.md lines 315-316 - step 8.5 numbering leaves step 9 unnumbered as step 10**

The plan inserts a new "step 8.5" between the existing steps 8 and 9 in the Implementer prompt, but does not instruct the implementer whether to renumber the existing step 9 to step 10. While decimal numbering preserves the existing step 9 label, the plan should explicitly state whether the existing `9. Return exactly one of:` line retains its number or is renumbered to `10.`.

## Changelog

### Review - 2026-03-26
- #1: Added Step 5 to extend the sub-task log template with a `Bash-used:` field (after `Re-read:`, before `Agent:`) for auditability and consistency with Grep/Re-read fields; updated verification item 4 to reference the log row instead of the Implementation field

### Prefect-1 - 2026-03-26
- [minor] Approach: corrected "initialized at MT-1" to "initialized to the empty set in the MT-3 state block at line 143" - USED_COMMANDS belongs in the MT-3 state block alongside other Commander state variables, not in MT-1 (M7-56-2-mission-command-tracking.md:11)

### Review #2 - 2026-03-26
- [minor] Step 3: replaced "in the same paragraph or as a new sentence immediately after the existing Implementer subagent block" with "Insert as a new standalone paragraph immediately after the Implementer subagent block (between line 319 and line 321 of SKILL.md)" - no prose exists between line 319 and line 321, so the insertion must create new text rather than append to existing prose
- [minor] Step 5: added explicit note that `Bash-used:` is intentionally excluded from the MT-3d enforcement gate (lines 348-379) because it is an audit field whose absence should not block task completion, unlike Agent/Re-read/Status/Implementation/Timestamp
- [nit] Step 2: clarified that the existing `9. Return exactly one of:` retains its step 9 number and is not renumbered to step 10 after inserting step 8.5
