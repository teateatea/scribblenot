## Task

#41 Confirm and fix mission-team task execution order to respect premission priority ranking

## Context

Sub-tasks 41.2 and 41.3 replaced the D-score tie-break in MT-2 and MT-3a with a TASK_LIST-position tie-break. This verification sub-task traces tasks #19, #39, #43, #47, #49, #50 through both the old and new ordering rules using live M6 evidence to confirm the fix would have produced the correct sequence.

## Approach

No file changes. Trace the six tasks through the old rule (D-score descending) and the new rule (TASK_LIST position ascending) and confirm the new rule places #19/#43/#47 before #39/#49/#50.

## Critical Files

- `C:/Users/solar/Documents/Claude Projects/scribblenot/pathfinder/MISSION-LOG-6-skill-log-quality.md` (premission order, actual execution timestamps, sub-task 41.1 findings)
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` (MT-2 line 133 and MT-3a line 145, updated by 41.2/41.3)

## Reuse

No implementation. Verification only.

## Steps

No implementation steps. All findings are documented in the Progress / Implementation section below.

## Progress / Implementation

### Input data

**Premission order** (from MISSION-LOG-6 Tasks line, positions 0-indexed):

| Task | TASK_LIST position | D score |
|------|--------------------|---------|
| #19  | 0                  | 10      |
| #43  | 1                  | 10      |
| #47  | 2                  | 15      |
| #39  | 9                  | 35      |
| #49  | 11                 | 45      |
| #50  | 15                 | 65      |

D scores sourced from CLOSED-TASKS.md entries for completed tasks.

### Old ordering rule (MT-2 and MT-3a: tie by D score descending)

All six tasks entered MT-2 at P:99 (no [P:N] annotations exist in TASKS.md). With all priorities equal, the old tie-break sorted by D score descending:

Sorted order: #50 (D:65), #49 (D:45), #39 (D:35), #47 (D:15), #19 (D:10), #43 (D:10)

For the #19 / #43 tie at D:10, MT-3a under the old rule also broke ties by D score - producing an indeterminate result between the two equal-D tasks rather than respecting premission order.

**Result**: #50, #49, #39 run first; #19, #43, #47 run last. This matches the actual M6 execution observed in the sub-task log timestamps: #50.1 at 19:18, #49.1 at 19:37, #39.1 at 20:33, and #19.1/#43.1 at 00:39 near mission end.

### New ordering rule (MT-2 and MT-3a: tie by TASK_LIST position ascending)

With all priorities still equal at P:99, the new tie-break sorts by TASK_LIST insertion position (lower index = higher priority):

Sorted order: #19 (pos:0), #43 (pos:1), #47 (pos:2), #39 (pos:9), #49 (pos:11), #50 (pos:15)

**Result**: #19, #43, #47 run first; #39, #49, #50 run after. This is the premission priority sequence.

### Confirmation

The new rule correctly places #19, #43, #47 before #39, #49, #50, reversing the M6 inversion.

The fix is sufficient for same-priority-tier cases (all tasks at P:99). Future missions where [P:N] annotations exist will still sort by explicit priority first; TASK_LIST position only applies within the same tier, which is the correct behavior.

### Residual gap (noted by sub-task 41.4)

Sub-task 41.4 confirmed that descending-integer PRIORITY_MAP values are not needed - the TASK_LIST-position tie-break alone is sufficient and avoids corrupting the X^2 decay algorithm (#37). No further changes are required.

## Verification

### Manual tests

1. Run `/pathfinder-mission-team MISSION-7-BRIEF` (or any future mission) where all tasks are at P:99 and no [P:N] annotations exist. Confirm in the resulting MISSION-LOG that the execution order of completed tasks matches the `## Task Priority Order` section of the BRIEF file.
2. Check that the sub-task log timestamps in the new mission run increase in the same sequence as the BRIEF priority list (allowing for re-queues due to failures).

### Automated tests

No automated test harness exists for SKILL.md ordering logic. A realistic option would be a shell script that feeds a synthetic TASK_LIST to a mock MT-2 reorder stub and asserts the output sequence matches the input position order rather than D-score descending. This would be low-value until the skill itself is refactored into testable units.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | by position in TASK_LIST (earlier = higher priority)`

### Verification result (2026-03-25)

Doc check PASSED. The phrase "by position in TASK_LIST (earlier = higher priority)" is present at line 133 of `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`.

- Old rule (D-score descending): executed #50, #49, #39 first; #19, #43, #47 last - matching the M6 inversion.
- New rule (TASK_LIST position ascending): executes #19, #43, #47 first; #39, #49, #50 after - matching premission priority order.
- Fix confirmed: the TASK_LIST-position tie-break corrects the M6 ordering inversion for same-priority tasks.

## Implementation
Complete - 2026-03-25
