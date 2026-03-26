# M7-63-3: Verify add-task Skill Step 4a via Scenario Trace

**Task**: #63 - Cross-reference PROJECT-TESTS.md criteria into task descriptions at creation time

**Context**: Sub-tasks 1 and 2 designed and implemented Step 4a in the add-task skill. This sub-task is a read-only trace-through of three scenarios using real data from the project, confirming that the updated SKILL.md produces correct output for every branch of Step 4a: match, no-match, and absent-file.

**Approach**: Trace each path using concrete task numbers and file content from `.claude/PROJECT-TESTS.md` and `.claude/TASKS.md`. No SKILL.md edits are performed unless a wording defect is discovered during the trace. Record any required wording adjustments as inline notes at the end of the plan.

**Critical Files**:
- `C:/Users/solar/.claude/skills/add-task/SKILL.md` - the updated skill (read only in this sub-task)
- `C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/PROJECT-TESTS.md` - source of criteria (read only)
- `C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/TASKS.md` - target file for entry writes (read only)

**Reuse**: No code changes. All verification is by manual trace.

**Steps**:

1. **Confirm the SKILL.md Step 4a text is present and internally consistent.**

   Read `add-task/SKILL.md` and verify:
   - `### Step 4a - Look up PROJECT-TESTS.md criteria` heading exists between Step 4 and Step 5.
   - Step 4a sub-clauses a-d match the design in M7-63-1 (colon-terminated heading pattern, silent skips, TESTS_FOR_N variable name).
   - Step 5 entry format block shows the `Tests:` block with `2-space` indentation for `Tests:` and `7-space` indentation for each criterion line.

2. **Trace path (a): match -- task #1.**

   Task #1 is present in both TASKS.md (under `## Code Quality`) and PROJECT-TESTS.md.

   Step 4a trace for N=1:

   a. Glob finds `.claude/PROJECT-TESTS.md` -- file exists, proceed.

   b. Scan for `## Task #1:` -- found on line 3 of PROJECT-TESTS.md: `## Task #1: Remove dead code warning for \`current_value\` on \`HeaderState\``

   c. Lines immediately following the heading before the next blank line or `##`:
      - `- [ ] The \`pub fn current_value()\` method has been deleted from \`src/sections/header.rs\``
      - `- [ ] \`cargo build\` completes with zero warnings (no dead_code warning for \`current_value\`)`

      Two `- [ ]` lines found -- not empty, proceed to d.

   d. TESTS_FOR_N contains both lines verbatim.

   Expected TASKS.md entry for task #1 (after Step 5 write):

   ```
   - [ ] **#1** Remove dead code warning for unused `current_value` on `HeaderState`
     [D:10 C:55] Delete or use the `pub fn current_value()` method in `src/sections/header.rs` ...
     Joseph: about that dead code clean up, I don't like that it pops up when I cargo run.
     Context: not specified
     Tests:
          - [ ] The `pub fn current_value()` method has been deleted from `src/sections/header.rs`
          - [ ] `cargo build` completes with zero warnings (no dead_code warning for `current_value`)
   ```

   Verification check: `Tests:` is indented 2 spaces; each criterion line is indented 7 spaces; criterion text is verbatim from PROJECT-TESTS.md including the `- [ ]` prefix.

   Note: Task #1 already exists in TASKS.md. This trace applies to any future invocation of `/add-task` for task number N=1 in a fresh project, or as a mental model for any task that does have a match. The trace is valid for any N that appears as `## Task #N:` in PROJECT-TESTS.md.

   Label note: The example above shows `Joseph:` because that is the label used in the existing TASKS.md entry for task #1 (written before the `Joseph-Raw:` label was introduced in task #9). New entries written by the current SKILL.md will use `Joseph-Raw:` for user-submitted tasks. The `Tests:` block format is unchanged by this label difference.

3. **Trace path (b): no-match -- task #44.**

   Task #44 (`Add /add-tasks as a forwarding alias`) is present in TASKS.md. Scan PROJECT-TESTS.md for `## Task #44:` -- no such heading exists in the file.

   Step 4a trace for N=44:

   a. Glob finds `.claude/PROJECT-TESTS.md` -- file exists, proceed.

   b. Scan for `## Task #44:` -- not found anywhere in the file.

   c. No heading found: skip silently per clause c.

   d. TESTS_FOR_N is empty / not set.

   Expected TASKS.md entry for task #44: written as the standard 4-line format with no `Tests:` block appended. No placeholder, no empty label.

   Correctness confirmed: Step 4a clause c says "skip silently" when no heading is found. Step 5 says "Omit the `Tests:` block entirely when TESTS_FOR_N is empty." Both clauses agree; no wording adjustment needed.

4. **Trace path (c): absent-file.**

   The SKILL.md Step 4a clause a instructs the skill to "Locate `.claude/PROJECT-TESTS.md` relative to the project root using Glob. If the file does not exist, skip this step silently -- no placeholder or empty section is added."

   Glob run against `pathfinder/PROJECT-TESTS.md` returns no results (confirmed: file does not live in `pathfinder/`). The correct path is `.claude/PROJECT-TESTS.md`.

   The instruction uses `.claude/PROJECT-TESTS.md` -- this matches the actual file location. The absent-file path activates only when the file genuinely does not exist (e.g. in a fresh project that has not yet created PROJECT-TESTS.md). In that case Glob returns no matches and the step exits with no output, no error, and no entry modification.

   Correctness confirmed: the path in the SKILL.md instruction and the actual file location are consistent. No wording adjustment needed for this path.

5. **Check for edge-case wording gaps.**

   After the three traces, review whether any clause in Step 4a could produce ambiguous behavior:

   - Clause b pattern `## Task #N:` correctly excludes `## Task #N-2:` because the colon immediately follows the digit (no hyphen between N and colon). Confirmed by checking PROJECT-TESTS.md: `## Task #53-2:` would NOT match a search for `## Task #53:` because the pattern requires the colon as the only character after the digit.
   - Clause c "zero `- [ ]` lines" guard prevents an empty `Tests:` block (e.g. if a heading exists with only prose or `- [x]` items). No such heading exists in the current file, but the guard is correct.
   - Batch adds: each task number has its own independent lookup. No cross-contamination is possible because TESTS_FOR_N is scoped per iteration.

   No wording adjustments required. All paths produce correct output as written.

**Verification**:

### Manual tests

- Invoke `/add-task` with a task that would receive N=1 (e.g. simulate by reading the current highest task number and confirming it is not 1, then manually note what criteria would appear). Open TASKS.md and confirm the `Tests:` block is present with the exact two criterion lines from PROJECT-TESTS.md `## Task #1:`.
- Invoke `/add-task` with a novel task description and observe the assigned number N. If PROJECT-TESTS.md does not contain `## Task #N:`, confirm no `Tests:` block or placeholder appears in TASKS.md.
- In a copy of the project with PROJECT-TESTS.md temporarily removed, invoke `/add-task` and confirm the skill completes normally with no error.

### Doc checks

`C:/Users/solar/.claude/skills/add-task/SKILL.md | contains | ### Step 4a - Look up PROJECT-TESTS.md criteria`
`C:/Users/solar/.claude/skills/add-task/SKILL.md | contains | TESTS_FOR_N`
`C:/Users/solar/.claude/skills/add-task/SKILL.md | contains | ## Task #N:`
`C:/Users/solar/.claude/skills/add-task/SKILL.md | contains | Tests:`

## Trace Findings

All three paths (match, no-match, absent-file) produce correct output per the SKILL.md Step 4a instructions as written. No wording adjustments were required. The colon-terminator in the heading pattern is the critical guard against false matches on `#N-2` headings; it is present and correct in the current SKILL.md text.

## Prefect-1 Report

### Nit

- **N1** [nit] `M7-63-3-add-task-verify.md:48,55` - The code block example in Step 2 shows `Joseph:` as the source label for task #1, matching the existing TASKS.md entry. However, the current SKILL.md uses `Joseph-Raw:` for user-submitted tasks (introduced in task #9). A reader comparing the trace example against new entries would see a label mismatch without explanation. Added a "Label note" paragraph after the verification check to prevent misreading the example as prescriptive for new entries.

All other cross-checks passed (heading names, variable names, indentation spec, criterion text verbatim match, path references, task #44 absence from PROJECT-TESTS.md, and duplicate-guard logic).

## Changelog

### Review - 2026-03-26
- N1: Added Label note paragraph after Step 2 verification check clarifying that existing task #1 uses `Joseph:` (pre-task-#9 label) while new entries use `Joseph-Raw:`
