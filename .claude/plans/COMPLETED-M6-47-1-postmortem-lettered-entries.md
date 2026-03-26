## Task

#47 - Use letters (A, B, C...) for post-mortem entries in mission log

## Context

Mission post-mortem entries in pathfinder mission logs are currently formatted with bullet dashes and bracketed kebab-case slugs (e.g. `- **[repeated-plan-failures]**: ...`). When reviewing logs or submitting post-mortem items as `/add-task` entries, this format creates visual ambiguity with task numbers (`#N`). Using lettered entries (A), B), C)...) makes each entry distinctly identifiable and removes any risk of confusion with numeric task IDs.

## Approach

Edit the Mission Post-Mortem Writer subagent prompt in `SKILL.md` (MT-4 step 4b) to replace the bullet-based entry template with a lettered A), B), C)... format. Update the template block, the prose description, the clean-mission fallback entry, and any example slugs to consistently use lettered labels. The subagent must be told explicitly to use A), B), C)... labels.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 521: prose describing entry format
  - Lines 523-524: template block (the `- **[Slug]**:` bullet pattern)
  - Line 527: slug prose / examples
  - Line 529: clean-mission fallback entry

## Reuse

No existing utilities to reuse. This is a pure prompt-text change inside SKILL.md.

## Steps

1. Read `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` lines 507-531 to confirm current text before editing.

2. Replace the prose line describing entry format (line 521):

```
- Process inefficiencies observed during this mission. Each entry is formatted as a ready-to-submit /add-task entry.
+ Process inefficiencies observed during this mission. Each entry is labeled with a letter (A), B), C)...) and formatted as a ready-to-submit /add-task entry.
```

3. Replace the template block entries (lines 523-524):

```
- - **[Slug]**: <one-sentence description of the inefficiency and its impact>
-   Suggested task: "<imperative-phrased task title>" — <one to two sentences of context explaining what change would fix it and why it matters>
+ A) **[Slug]**: <one-sentence description of the inefficiency and its impact>
+    Suggested task: "<imperative-phrased task title>" — <one to two sentences of context explaining what change would fix it and why it matters>
```

4. Replace the clean-mission fallback entry (line 529):

```
- If no inefficiencies are identifiable (clean mission, zero denials, zero abandonments, no retry loops), append the section with a single entry: `- **[clean-mission]**: No process inefficiencies identified.`
+ If no inefficiencies are identifiable (clean mission, zero denials, zero abandonments, no retry loops), append the section with a single entry: `A) **[clean-mission]**: No process inefficiencies identified.`
```

## Verification

### Manual tests

- Open a recent mission log that contains a `## Mission Post-Mortem` section and confirm existing entries used bullet format (pre-change baseline).
- After the change, trigger a test mission completion (or manually invoke a Mission Post-Mortem Writer subagent with a sample log) and verify the appended post-mortem section uses `A)`, `B)`, `C)` labels instead of `- **[...]**:` bullets.
- Confirm the clean-mission fallback also uses `A)` label when no inefficiencies are found.

### Automated tests

No automated test infrastructure exists for SKILL.md prompt content. A realistic option would be a shell script that greps the updated SKILL.md for the pattern `A\) \*\*\[` and asserts it is present while `^> - \*\*\[` is absent, confirming the format was changed.

## Progress

- Step 1: Read lines 507-531 of SKILL.md; confirmed current bullet-based template text matches plan exactly.
- Step 2: Updated prose line (line 521) to mention lettered A), B), C)... format.
- Step 3: Updated template block entries (lines 523-524) from `- **[Slug]**:` to `A) **[Slug]**:`.
- Step 4: Updated clean-mission fallback (line 529) from `- **[clean-mission]**:` to `A) **[clean-mission]**:`.

## Implementation
Complete -- 2026-03-25

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | A) **[Slug]**:`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | A) **[clean-mission]**:`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | - **[Slug]**:`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | - **[clean-mission]**:`
