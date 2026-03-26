## Task

#60 - Recompute Current ETA on each new top-level task start (MT-3a)

## Context

The mission log's "Current Estimated Completion Time" line is only set once at initialization (MT-1 step 2c). As tasks complete and COMPLETED_D grows, the remaining work shrinks, but the displayed ETA never updates. This makes the ETA increasingly stale and misleading as the mission progresses. The fix is to recompute the ETA in MT-3a each time a new top-level task is selected (not per sub-task), overwriting only the "Current Estimated Completion Time" line with a fresh estimate and the time of the update.

## Approach

After MT-3a selects the next task and initializes PLAN_FILES and PRIOR_ATTEMPT_MAP, insert an ETA recomputation block. The block reads the current wall-clock time via Bash, computes remaining_D = T - COMPLETED_D, derives a new ETA using the same 0.43 factor as the initial estimate, then uses the Edit tool to overwrite the "Current Estimated Completion Time" line in MISSION_LOG_PATH. The "Initial Estimated Completion Time" line is never touched.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` lines 151-155 (MT-3a section)

## Reuse

- Existing formula `round(T * 0.43)` from MT-1 step 2b and 2c - same factor applied to remaining_D
- Existing `TZ=America/Toronto date` Bash pattern already used in MT-3c step 5 and MT-3f
- Existing Edit tool pattern used throughout the skill for in-place MISSION-LOG updates

## Steps

1. In `SKILL.md`, locate the MT-3a section (lines 151-155). After the line "Initialize `PLAN_FILES[task] = []` and `PRIOR_ATTEMPT_MAP[task] = []` for the selected task before entering MT-3b.", insert the following new paragraph:

```
After initializing PLAN_FILES and PRIOR_ATTEMPT_MAP, recompute the Current ETA:

1. Run `TZ=America/Toronto date +"%H:%M"` and store the result as `NOW_HH_MM`.
2. Compute `remaining_D = T - COMPLETED_D`.
3. Compute `eta_offset_minutes = round(remaining_D * 0.43)`.
4. Parse `NOW_HH_MM` into total minutes from midnight: `now_total = HH * 60 + MM`.
5. Compute `eta_total = (now_total + eta_offset_minutes) % 1440`.
6. Format `new_eta = zero-padded(eta_total / 60):zero-padded(eta_total % 60)` (e.g. `14:07`).
7. Use the Edit tool to overwrite the "Current Estimated Completion Time" line in MISSION_LOG_PATH:
   - old: `- Current Estimated Completion Time: <any text>`
   - new: `- Current Estimated Completion Time: <new_eta> (Updated at <NOW_HH_MM>)`
   This replaces only the Current line; the Initial Estimated Completion Time line is never modified.
```

Diff of the MT-3a section:

```
 #### MT-3a: Pick next task

 Select the highest-priority unblocked task (all its dependencies are complete). On a tie, pick the one with the highest PRIORITY_MAP score. If all remaining tasks are blocked by incomplete dependencies, pick the blocked task whose dependencies are furthest along (most sub-tasks complete).

 Initialize `PLAN_FILES[task] = []` and `PRIOR_ATTEMPT_MAP[task] = []` for the selected task before entering MT-3b.
+
+After initializing PLAN_FILES and PRIOR_ATTEMPT_MAP, recompute the Current ETA:
+
+1. Run `TZ=America/Toronto date +"%H:%M"` and store the result as `NOW_HH_MM`.
+2. Compute `remaining_D = T - COMPLETED_D`.
+3. Compute `eta_offset_minutes = round(remaining_D * 0.43)`.
+4. Parse `NOW_HH_MM` into total minutes from midnight: `now_total = HH * 60 + MM`.
+5. Compute `eta_total = (now_total + eta_offset_minutes) % 1440`.
+6. Format `new_eta = zero-padded(eta_total / 60):zero-padded(eta_total % 60)` (e.g. `14:07`).
+7. Use the Edit tool to overwrite the "Current Estimated Completion Time" line in MISSION_LOG_PATH:
+   - old: `- Current Estimated Completion Time: <any text>`
+   - new: `- Current Estimated Completion Time: <new_eta> (Updated at <NOW_HH_MM>)`
+   This replaces only the Current line; the Initial Estimated Completion Time line is never modified.

 #### MT-3b: Sub-task decomposition
```

## Verification

### Manual tests

- Start a mission with at least 2 tasks. After task 1 completes and MT-3a picks task 2, open MISSION_LOG_PATH and confirm the "Current Estimated Completion Time" line shows a new HH:mm value and "Updated at HH:mm" that differs from the initial "Started at HH:mm" timestamp.
- Confirm the "Initial Estimated Completion Time" line is unchanged from its original value set in MT-1.
- For a single-task mission, verify that MT-3a still runs the ETA update (even if remaining_D = T, since COMPLETED_D = 0 at the first pick).

### Automated tests

- Unit test (no wrap): given T=10, COMPLETED_D=4, NOW_HH_MM="23:50", verify remaining_D=6, eta_offset=round(6*0.43)=3, now_total=1430, eta_total=(1430+3)%1440=1433, new_eta="23:53".
- Unit test (midnight wrap): given T=25, COMPLETED_D=0, NOW_HH_MM="23:50", verify remaining_D=25, eta_offset=round(25*0.43)=11, now_total=1430, eta_total=(1430+11)%1440=1441%1440=1, new_eta="00:01".
- Doc check: after the edit, verify MISSION_LOG_PATH contains "Current Estimated Completion Time" with "Updated at" and does not duplicate the Initial line.

### Doc checks

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | After initializing PLAN_FILES and PRIOR_ATTEMPT_MAP, recompute the Current ETA`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | remaining_D = T - COMPLETED_D`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Current Estimated Completion Time: <new_eta> (Updated at <NOW_HH_MM>)`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Initial Estimated Completion Time line is never modified`

## Changelog

### Review - 2026-03-26
- #1: Fixed unit test arithmetic - the original example claimed eta_total=(1430+3)%1440=13 and new_eta="00:13", but 1433%1440=1433 (not 13, no wrap occurs). Replaced with two correct tests: a no-wrap case (now_total=1430, offset=3, eta=1433 -> "23:53") and a midnight-wrap case (T=25, offset=11, eta=1%1440=1 -> "00:01").
