## Task

#25 - Add "Context at finish:" field to Mission Complete section in MISSION-LOG template

## Context

The ## Mission Complete section in pathfinder-mission-team (MT-4 step 4) has no field for recording main-instance context usage at the end of a mission. Joseph wants to record context % after each mission to correlate it against mission difficulty over time.

## Approach

Append a `- Context at finish:` line to the ## Mission Complete section template in `pathfinder-mission-team/SKILL.md`, immediately after the `- Duration: <DURATION>` line. No other changes are needed; the field is left blank for the user to fill in manually.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` — lines 386-396 (MT-4 step 4 template)

## Reuse

No existing utilities to reuse. Single-line addition to a markdown template.

## Steps

1. Edit `pathfinder-mission-team/SKILL.md`: add `- Context at finish:` after the Duration line in the MT-4 step 4 template.

```diff
 - End-Time: <END_TIME>
 - Duration: <DURATION>
+- Context at finish:
 ```

## Verification

### Manual tests

- Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and confirm the ## Mission Complete template (inside the MT-4 step 4 markdown block) contains `- Context at finish:` on the line immediately after `- Duration: <DURATION>`.
- Run a pathfinder mission to completion and verify the generated MISSION-LOG ## Mission Complete section includes a blank `- Context at finish:` line.

### Automated tests

No test runner applies (this is a skill `.md` file, not compiled code). The doc check below covers automated verification.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | - Context at finish:`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | - Duration: <DURATION>`

## Progress

- Step 1: Added `- Context at finish:` line after `- Duration: <DURATION>` in the MT-4 step 4 template in pathfinder-mission-team/SKILL.md
