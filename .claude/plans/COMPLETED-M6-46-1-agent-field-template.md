## Task

#46 - Add Agent field to MT-3c sub-task log template

## Context

The MT-3c sub-task log template in the pathfinder-mission-team SKILL.md records Status, TDD, Reviewers, Prefects, Implementation, Shim-removal, and Timestamp for each sub-task. There is currently no field indicating whether the sub-task was executed by a subagent or by the Mission Commander directly (main context). Adding an Agent field makes log entries more auditable and helps diagnose failures by distinguishing delegation patterns.

## Approach

Insert a single new line `- Agent: subagent | main` into the template block at MT-3c step 5, placed after `- Shim-removal:` and before `- Timestamp:`. The Mission Commander fills this in at log-write time: use `subagent` if the sub-task's Implementer was delegated to a spawned Sonnet subagent (the normal path), and `main` if the Mission Commander ran the implementation directly.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` lines 299-310 (MT-3c step 5, the sub-task log template block)

## Reuse

No existing utilities needed. This is a pure template text edit.

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`. Locate the sub-task log template block inside MT-3c step 5 (currently at lines 301-310). The block currently ends with:

```
- Shim-removal: <"N/A" if no shim was introduced | "Confirmed: <what was removed>" if a shim was removed | "Absent" if a shim was introduced but no removal confirmation was logged>
- Timestamp: <SUBTASK_TIME>
```

Apply the following edit (insert one line between Shim-removal and Timestamp):

```diff
 - Shim-removal: <"N/A" if no shim was introduced | "Confirmed: <what was removed>" if a shim was removed | "Absent" if a shim was introduced but no removal confirmation was logged>
+- Agent: <subagent | main> (subagent = delegated to a spawned Sonnet subagent; main = run directly by Mission Commander)
 - Timestamp: <SUBTASK_TIME>
```

The full updated template block becomes:

```
### Sub-task N.<SUB_ID>: <description>
- Status: Pass / Fail / Blocked
- TDD: <TESTS WRITTEN file:line> / (no tests) / FAILED: <reason>
- Reviewers: <N>
- Prefects: <N>
- Implementation: <summary>
- Shim-removal: <"N/A" if no shim was introduced | "Confirmed: <what was removed>" if a shim was removed | "Absent" if a shim was introduced but no removal confirmation was logged>
- Agent: <subagent | main> (subagent = delegated to a spawned Sonnet subagent; main = run directly by Mission Commander)
- Timestamp: <SUBTASK_TIME>
```

No other changes to the file.

## Verification

### Manual tests

- Read the updated SKILL.md and confirm the template block contains exactly the fields: Status, TDD, Reviewers, Prefects, Implementation, Shim-removal, Agent, Timestamp in that order.
- Confirm `- Agent:` appears after `- Shim-removal:` and before `- Timestamp:`.
- Confirm no double blank lines were introduced around the template block.

### Automated tests

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | - Agent: <subagent | main>`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | - Timestamp: <SUBTASK_TIME>`

### Doc checks

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | - Agent: <subagent | main>`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Shim-removal:`
