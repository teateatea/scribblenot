# M7-63-1: Document PROJECT-TESTS.md Structure and Matching Heuristics

**Task**: #63 - Cross-reference PROJECT-TESTS.md criteria into task descriptions at creation time

**Context**: Sub-task 1 of task #63 is a research and design sub-task. Task #63 asks the add-task skill to read PROJECT-TESTS.md and append any criteria from a matching section to the new TASKS.md entry. Before implementing that in sub-task 2, this sub-task documents the exact structure of PROJECT-TESTS.md, its matching heuristics, and the text to extract.

**Approach**: Analyze the already-read PROJECT-TESTS.md, document its structure and matching logic, and specify the extraction and insertion format for use in sub-task 2. No file edits are performed in this sub-task.

**Critical Files**:
- `C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/PROJECT-TESTS.md` (source of truth, read-only in this sub-task)
- `C:/Users/solar/.claude/skills/add-task/SKILL.md` (target skill to be modified in sub-task 2)

**Reuse**: No code to reuse; this is a documentation and design sub-task.

**Steps**:

1. **Confirm PROJECT-TESTS.md exists and note its location.**

   File is present at `.claude/PROJECT-TESTS.md` relative to project root.

2. **Document the file structure.**

   PROJECT-TESTS.md uses a flat Markdown structure with one `##` heading per task. Each heading has the exact format:

   ```
   ## Task #N: <short description>
   ```

   Immediately beneath each heading is a bullet list of acceptance criteria, each beginning with `- [ ]`. Every criterion is one Markdown list item and may span one line only in the current file. There are no sub-headings, code blocks, or nested lists. Criteria never start with `- [x]` (there are no pre-checked items). A blank line separates consecutive task sections.

   Example (from the actual file):

   ```
   ## Task #1: Remove dead code warning for `current_value` on `HeaderState`
   - [ ] The `pub fn current_value()` method has been deleted from `src/sections/header.rs`
   - [ ] `cargo build` completes with zero warnings (no dead_code warning for `current_value`)
   ```

3. **Identify all fields a criterion entry contains.**

   Each criterion entry has exactly two fields:
   - **Task number** (N): extracted from the `## Task #N` heading line.
   - **Criterion text**: the full text of the `- [ ] ...` line, including the checkbox prefix.

   There are no additional fields (no priority, owner, or status beyond the `[ ]` checkbox).

4. **Design the matching heuristic.**

   Matching is by task number only (as specified by the task #63 acceptance criteria in PROJECT-TESTS.md itself):

   > Matching is by task number: criteria are copied only if PROJECT-TESTS.md contains a ## Task #N section for that exact task number.

   Algorithm for sub-task 2:

   a. After add-task assigns a task number N (Step 4 of the skill), read PROJECT-TESTS.md.
   b. Scan for a heading matching the regex `^## Task #<N>:` (exact integer match; do not match #10 when searching for #1).
   c. If no such heading exists, skip silently -- no placeholder or empty section is added to the TASKS.md entry.
   d. If a heading matches, collect all consecutive `- [ ]` lines immediately following it (stop at the next blank line or the next `##` heading). If no `- [ ]` lines are found (heading exists but section is empty), treat as if no heading matched and skip silently.
   e. Append the collected criteria to the TASKS.md entry under a `Tests:` label (see Step 5 below for the exact format).

5. **Specify the text to extract and its insertion format.**

   Text to extract: the raw `- [ ] ...` lines verbatim, preserving checkbox and whitespace exactly as they appear in PROJECT-TESTS.md.

   Insertion position in TASKS.md entry: after the existing four-line entry block (brief, scores+interpretation, source, context), append a `Tests:` block. Example:

   ```
   - [ ] **#1** Remove dead code warning for unused `current_value` on `HeaderState`
     [D:10 C:55] Delete or use the `pub fn current_value()` method ...
     Joseph: about that dead code clean up ...
     Context: not specified
     Tests:
       - [ ] The `pub fn current_value()` method has been deleted from `src/sections/header.rs`
       - [ ] `cargo build` completes with zero warnings (no dead_code warning for `current_value`)
   ```

   The `Tests:` label is indented with two spaces to align with the other continuation lines. Each criterion line is indented with four spaces (two more than `Tests:`). This preserves the block structure of the TASKS.md entry and makes the criteria visually distinct from the label.

6. **Edge cases to handle in sub-task 2.**

   - Task number N is a fresh assignment (not yet in TASKS.md), so the PROJECT-TESTS.md lookup happens after number assignment but before the file write.
   - Duplicate tasks (sub-entries #M-2) should look up the parent task number M, not the suffixed number. Note: PROJECT-TESTS.md itself uses the `-2` suffix convention (e.g. `## Task #53-2`). If a sub-entry `#M-2` is being added and PROJECT-TESTS.md contains BOTH `## Task #M` and `## Task #M-2`, the lookup uses parent `M` only -- the `-2`-suffixed section in PROJECT-TESTS.md is treated as a separate, standalone task entry and is not used for sub-entry lookup.
   - If PROJECT-TESTS.md does not exist at the expected path, skip the lookup step silently (no error, no placeholder).
   - Batch adds (multiple tasks in one call) each receive their own independent lookup; criteria from one task are never appended to another.

**Verification**:

### Manual tests

- Run `/add-task` with a task that has a matching number in PROJECT-TESTS.md and confirm the `Tests:` block appears in TASKS.md.
- Run `/add-task` with a task number absent from PROJECT-TESTS.md and confirm no `Tests:` block or placeholder appears.
- Verify that task #1 (which exists in PROJECT-TESTS.md) produces the correct two criteria lines when added.

### Additional manual integration tests

- Given a sample PROJECT-TESTS.md string and a target task number, manually verify (by reading TASKS.md after the run) that the extraction logic returns the correct list of criterion lines.
- Invoke the add-task skill in sequential mode with a known task number and confirm the written TASKS.md entry contains the expected `Tests:` block verbatim.

## Prefect-1 Report

### Issues Fixed

1. **[minor] M7-63-1-project-tests-structure.md Step 4d** - Empty-criteria edge case unhandled. The algorithm skipped silently when no heading was found (Step 4c), but had no guard for the case where a heading exists with zero `- [ ]` lines following it. Step 4e would then append an empty `Tests:` label with no content. Fixed by adding an explicit skip-silently clause to Step 4d when the collected list is empty.

## Prefect-2 Report

### Issues Found

1. **[blocking] M7-63-1-project-tests-structure.md:56 (Step 4b)** - Regex `^## Task #<N>(\b|:)` incorrectly matches `## Task #N-2` headings. The `-` character is a non-word character, so `\b` fires between a digit and `-`, causing a search for N=53 to also match `## Task #53-2`. Since PROJECT-TESTS.md contains entries like `## Task #53-2`, this produces false positives. The fix is to use `:` as the only terminator (all headings in PROJECT-TESTS.md use `## Task #N: description` format), changing the regex to `^## Task #<N>:`. This also resolves the interaction noted in Step 6: the sub-entry lookup for parent M=53 would incorrectly pull criteria from `## Task #53-2` under the current regex.

2. **[minor] M7-63-1-project-tests-structure.md:67-77 (Step 5)** - The prose at line 77 states "`Tests:` is indented with two spaces" and "each criterion line is indented with four spaces," but the example code block (which is itself indented 3 spaces inside a Markdown list) shows `Tests:` and `- [ ]` lines at the same visual indentation level. An implementer reading sub-task 2 cannot determine the intended indentation from the example alone. The code block should show the exact bytes (e.g., use `·` to mark spaces, or provide a raw string with explicit counts) or be moved to a top-level fenced block with no surrounding list indentation so the spaces are unambiguous.

## Changelog

### Review - 2026-03-26
- #1 (minor): Step 6 edge case for sub-entries clarified to acknowledge that PROJECT-TESTS.md itself uses the `-2` suffix convention; added explicit rule that sub-entry lookup uses parent M only, and that `-2`-suffixed sections in PROJECT-TESTS.md are standalone entries not used for sub-entry lookup.
- #2 (nit): Renamed "Automated tests" heading to "Additional manual integration tests" and reworded items to reflect that add-task is a skill invoked manually, not a unit-testable codebase with a test runner.

### Review - 2026-03-26 (Prefect-1)
- #1 (minor): Step 4d -- added explicit empty-list guard: if heading matches but no `- [ ]` lines follow, skip silently rather than appending an empty `Tests:` block.

### Review - 2026-03-26 (Prefect-2)
- #1 (blocking): Step 4b -- changed regex from `^## Task #<N>(\b|:)` to `^## Task #<N>:` to prevent false match of `## Task #53-2` when searching for N=53; colon is the only valid terminator in the file's heading format.
- #2 (minor): Step 5 code block -- corrected criterion indentation from 5 spaces (same level as Tests:) to 7 spaces (4 spaces relative to the entry line, 2 more than Tests:); updated prose to clarify "two more than Tests:" so implementer has unambiguous byte counts.
