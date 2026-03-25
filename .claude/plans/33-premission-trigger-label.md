## Task
#33 - Add second clarification threshold to premission (D > 50, C < 70)

## Context
PM-1.5 already labels each clarification candidate with `[D:<d> C:<c>, delta=<delta>]`, but gives no indication of *why* that task was flagged. With two separate trigger conditions now in play (`delta > 0` and `D > 50 & C < 70`), the user reading the AskUserQuestion prompt cannot tell which rule fired. Showing the trigger reason inline makes the prompt self-explanatory and helps the user calibrate their answer.

## Approach
Extend the format string in PM-1.5 to append a short trigger label after the existing `[D:<d> C:<c>, delta=<delta>]` bracket. Compute the label at classification time: if both conditions fired, show both; if only one fired, show that one. The label uses compact notation to keep the header line short.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` — lines 47-53 (PM-1.5 format block)

## Reuse
The delta and D/C values are already computed in PM-1 step 5 and carried into PM-1.5; no new data needs to be derived.

## Steps
1. Open `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` and locate the PM-1.5 format line (line 49):

```
- "Task #<N> [D:<d> C:<c>, delta=<delta>] - <title>
```

2. Replace that line with a version that appends a trigger label:

```diff
-> "Task #<N> [D:<d> C:<c>, delta=<delta>] - <title>
+> "Task #<N> [D:<d> C:<c>, delta=<delta>, trigger=<trigger>] - <title>
```

Where `<trigger>` is resolved as follows (evaluate both conditions independently):
- If `delta > 0` only: `trigger=delta>0`
- If `D > 50 & C < 70` only: `trigger=D>50&C<70`
- If both conditions are true: `trigger=delta>0,D>50&C<70`

3. Add a one-sentence note directly after the format block (before the "Only include Q2..." line) explaining the trigger values:

```diff
 > Q3: <third targeted question, if warranted>"
+
+`<trigger>` is one of `delta>0`, `D>50&C<70`, or `delta>0,D>50&C<70` depending on which condition(s) caused this task to be flagged.

 Only include Q2 and Q3 when genuinely needed ...
```

## Verification

### Manual tests
- Invoke `/pathfinder-premission` against a task list that contains:
  - One task where only `delta > 0` (e.g. D=40, C=30)
  - One task where only `D > 50 & C < 70` (e.g. D=60, C=65)
  - One task where both conditions fire (e.g. D=80, C=50)
- Confirm the AskUserQuestion prompt for each candidate shows the correct `trigger=` label.
- Confirm fast-path tasks (delta <= 0 and not in the D>50&C<70 zone) do not appear in PM-1.5 at all.

### Automated tests
- No automated test harness exists for skill SKILL.md files; the manual test above is the primary verification path.
- A future unit test could parse the AskUserQuestion call arguments and assert the `trigger=` substring matches the expected label for a given D/C pair.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | trigger=<trigger>`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | delta>0,D>50&C<70`

## Changelog

### Review - 2026-03-25
- #1: Fixed Step 3 diff context mismatch - diff now shows the note inserted before "Only include Q2..." (using the closing format-block line as context) rather than after it, matching the prose instruction.

### Review #2 - 2026-03-25
- #1 (nit): Corrected Step 3 diff context to include the pre-existing blank line (SKILL.md line 53) between `> Q3...` and `Only include Q2...` so the diff applies cleanly against the actual file.

### Review #3 - 2026-03-25
- #1 (blocking): Added missing `> ` blockquote prefix to both `-` and `+` lines in the Step 2 diff - SKILL.md line 49 begins with `> "Task...` so the diff as written would not match the actual file content.

## Progress
- Step 1: Located PM-1.5 format line in pathfinder-premission/SKILL.md
- Step 2: Updated format line to include `trigger=<trigger>` field in the bracket
- Step 3: Added one-sentence note after the format block explaining trigger values
