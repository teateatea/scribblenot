# M7-56-3-postmortem-dp-recommendations

## Task

#56-2 - Track per-mission command hit counts in DEFAULT-PERMISSIONS; add post-mortem recommendation section

## Context

The Mission Post-Mortem Writer subagent (step 4b of MT-4 in pathfinder-mission-team SKILL.md) currently only identifies process inefficiencies as /add-task entries. It does not inspect which commands were used during the mission against the DEFAULT-PERMISSIONS baseline to surface candidates for promotion. As DEFAULT-PERMISSIONS grows over time, the post-mortem is the natural moment to catch commands that proved essential but are not yet in the baseline, preventing them from being silently omitted from future missions.

## Approach

Extend the Mission Post-Mortem Writer's prompt (step 4b, lines 530-554 of SKILL.md) with two targeted edits: (1) add DEFAULT-PERMISSIONS.json to the subagent's "Read:" list so it has access to the baseline, and (2) extend the closing instructions to append a second output section `## Default Permissions Recommendations`. The subagent will scan the mission log to reconstruct which Bash commands were used, diff those against `approved_actions[].pattern` entries in DEFAULT-PERMISSIONS.json, and list any gaps as promotion candidates with a one-sentence justification each. If all used commands are already covered (or DEFAULT-PERMISSIONS.json is absent), it writes a "none" note instead.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - lines 530-554 (Mission Post-Mortem Writer subagent prompt; edits touch lines 534-554)

## Reuse

- The mission log (already written to disk at `<MISSION_LOG_PATH>`) contains the Sub-task Log and Bash invocation records, which serve as the subagent's source for reconstructing which commands were used. The outer runner's USED_COMMANDS variable is not directly accessible to a spawned subagent.
- The existing DEFAULT-PERMISSIONS.json read pattern from step 4c (lines 556-561) shows the correct file path token: `<PROJECT_ROOT>/pathfinder/DEFAULT-PERMISSIONS.json`.

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and locate the Mission Post-Mortem Writer block (lines 530-554). The two edits in Step 2 target lines 534-536 and 539-554 respectively.

2. Make two targeted edits inside the subagent prompt block in `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`.

**Edit A** — Add DEFAULT-PERMISSIONS.json to the subagent's "Read:" list (lines 534-536). Insert one line after the existing bullet:

```diff
  > Read:
  > - `<MISSION_LOG_PATH>` (full mission log — Sub-task Log, Permission Denials, Abandonment Records, and any Prefect Issues notes)
+ > - `<PROJECT_ROOT>/pathfinder/DEFAULT-PERMISSIONS.json` (if it exists; skip if absent)
  >
```

**Edit B** — Replace the closing instructions paragraph and return-value line (lines 539-554) with the updated version that appends both sections. The diff context lines (`  >`) are shown for orientation:

```diff
  > Identify process inefficiencies: repeated failures, permission denials, subagent confusion, plan-review loops that took more passes than expected, abandonment cycles, or any pattern that a future task could address by changing the mission skill or project config.
  >
-> Append the following section to `<MISSION_LOG_PATH>` (always append this section, even if the list has only one entry):
->
-> ```markdown
-> ## Mission Post-Mortem
->
-> Process inefficiencies observed during this mission. Each entry is labeled with a letter (A), B), C)...) and formatted as a ready-to-submit /add-task entry.
->
-> A) **[Slug]**: <one-sentence description of the inefficiency and its impact>
->    Suggested task: "<imperative-phrased task title>" — <one to two sentences of context explaining what change would fix it and why it matters>
-> ```
->
-> Use a short bracketed slug (2-4 words, kebab-case) that names the pattern, e.g. [repeated-plan-failures], [permission-denial-loop], [abandonment-thrash].
->
-> If no inefficiencies are identifiable (clean mission, zero denials, zero abandonments, no retry loops), append the section with a single entry: `A) **[clean-mission]**: No process inefficiencies identified.`
->
-> Do NOT use AskUserQuestion. Return exactly: `POST-MORTEM WRITTEN: <N> inefficiency/inefficiencies noted`
+> Append the following two sections to `<MISSION_LOG_PATH>` (always append both sections):
+>
+> ```markdown
+> ## Mission Post-Mortem
+>
+> Process inefficiencies observed during this mission. Each entry is labeled with a letter (A), B), C)...) and formatted as a ready-to-submit /add-task entry.
+>
+> A) **[Slug]**: <one-sentence description of the inefficiency and its impact>
+>    Suggested task: "<imperative-phrased task title>" — <one to two sentences of context explaining what change would fix it and why it matters>
+>
+> ## Default Permissions Recommendations
+>
+> Commands used this mission that are not yet covered by an entry in DEFAULT-PERMISSIONS.json. Each entry is a promotion candidate with written justification.
+>
+> - `<bash pattern>` — <one sentence explaining why this command is essential and what problems adding it to DEFAULT-PERMISSIONS would prevent>
+> ```
+>
+> Use a short bracketed slug (2-4 words, kebab-case) that names the pattern, e.g. [repeated-plan-failures], [permission-denial-loop], [abandonment-thrash].
+>
+> If no inefficiencies are identifiable (clean mission, zero denials, zero abandonments, no retry loops), append the `## Mission Post-Mortem` section with a single entry: `A) **[clean-mission]**: No process inefficiencies identified.`
+>
+> For `## Default Permissions Recommendations`: read `<PROJECT_ROOT>/pathfinder/DEFAULT-PERMISSIONS.json` (already read above). Scan the mission log's Sub-task Log and Bash invocation records to reconstruct the list of Bash command patterns actually used this mission. Compare those patterns against `approved_actions[].pattern` values. For each used pattern NOT already covered by any entry in `approved_actions`, add a bullet with the pattern and a one-sentence justification. If DEFAULT-PERMISSIONS.json was absent or all used commands are already covered, write: `No new promotion candidates — all used commands are already in DEFAULT-PERMISSIONS.`
+>
+> Do NOT use AskUserQuestion. Return exactly: `POST-MORTEM WRITTEN: <N> inefficiency/inefficiencies noted, <M> promotion candidate(s)`
```

## Verification

### Manual tests

- Run a pathfinder mission (or simulate MT-4 completion) and confirm the Mission Post-Mortem Writer appends both `## Mission Post-Mortem` and `## Default Permissions Recommendations` sections to the mission log.
- Verify that when a Bash command was used during the mission but its pattern is absent from DEFAULT-PERMISSIONS.json, it appears as a bullet in `## Default Permissions Recommendations`.
- Verify that when all used commands are already in DEFAULT-PERMISSIONS.json, the section contains the "No new promotion candidates" note instead of bullets.
- Verify the subagent return value now includes the promotion candidate count, e.g. `POST-MORTEM WRITTEN: 2 inefficiencies noted, 1 promotion candidate(s)`.

### Automated tests

- No automated test harness exists for the SKILL.md prompts; a scripted smoke test could mock USED_COMMANDS and DEFAULT-PERMISSIONS.json with known overlap/gap and assert the expected section content appears in the log output.

### Doc checks

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | ## Default Permissions Recommendations`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | promotion candidate(s)`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | No new promotion candidates`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | Do NOT use AskUserQuestion. Return exactly: \`POST-MORTEM WRITTEN: <N> inefficiency/inefficiencies noted\``

## Changelog

### Review - 2026-03-26
- #1 (blocking): Split single diff block into two targeted edits (Edit A / Edit B) - Edit A adds DEFAULT-PERMISSIONS.json to the subagent "Read:" list; without this the subagent had no explicit instruction to read the file.
- #2 (minor): Clarified that the subagent derives used commands from the mission log (Sub-task Log / Bash records) rather than accessing the outer runner's USED_COMMANDS variable, which is not accessible to spawned subagents.
- #3 (minor): Added "if DEFAULT-PERMISSIONS.json was absent" to the "No new promotion candidates" fallback condition so the missing-file edge case is handled.
- #4 (nit): Updated Approach, Critical Files, Step 1, and Reuse sections to reflect the expanded edit scope (lines 534-554) and corrected USED_COMMANDS sourcing description.
