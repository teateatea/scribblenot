# Project Foundation

## Goals
Mission 7 hardens the pathfinder skill system by closing the recurring failure patterns identified in Mission 6's post-mortem: permission-hook denials from compound commands and tilde paths, single-file fixes missing sibling files, and log-field enforcement gaps. It also improves mission observability (start times, ETA fields, active-log truncation, compact-event mirroring) and refines premission UX (priority ordering, duration estimation, command tracking, and sub-entry format disambiguation).

## Requirements
- All changes are confined to pathfinder skill files (SKILL.md, hooks, associated scripts) and supporting data files (DEFAULT-PERMISSIONS, MISSION-LOG templates)
- No changes to scribblenot application source code (src/, data/) unless a task explicitly targets it
- Each task must be implemented, reviewed by MT-3d, and have a passing log-field check before being marked complete
- All mv/git add operations in MT-3d must use individual commands, not compound bash
- Subagent prompts must reference C:/Users/solar/.claude as the literal path, not ~
- MISSION-LOG-active.md must be truncated at mission end before the next mission begins

## Task Priority Order
- #64 - Add multi-file pattern search to Implementer prompt
- #66 - Replace tilde paths with absolute paths in subagent prompts
- #65 - Rewrite MT-3d plan-rename step to individual mv + git add
- #68 - Upgrade MT-3d Status/Implementation/Timestamp check to hard block
- #69 - Truncate MISSION-LOG-active.md at mission end
- #67 - Store premission rank in PRIORITY_MAP; use as primary sort key in MT-2 and MT-3a
- #63 - Cross-reference PROJECT-TESTS.md criteria into task descriptions at creation time
- #59 - Mirror PreCompact hook entries to the numbered MISSION-LOG file
- #56 - Log command usage per mission; add Default Permissions baseline pulled into each premission
- #55 - Track premission duration and show estimate before committing to session
- #58 - Resolve collision between TASKS.md sub-entry format (#N-2) and pathfinder sub-task nomenclature
- #60 - Add Initial and Current Estimated Completion Time fields to MISSION-LOG Task Status
- #56-2 - Track per-mission command hit counts in DEFAULT-PERMISSIONS; add post-mortem recommendation section
- #57 - Fix M6 Start-Time recorded ~4 hours ahead of actual local time
- #61 - Add remaining count to Difficulty field in MISSION-LOG mission section
- #62 - Omit (P:99) priority annotation from Tasks list in MISSION-LOG Mission section

## Explicit Non-Goals
- Do not modify scribblenot application code (src/, data/, Cargo.toml) unless a task explicitly targets it
- Do not redesign the pathfinder skill architecture or change the MT-1 through MT-4 phase structure beyond what individual tasks require
- Do not add new mission-team phases or rename existing phase labels
- Do not create new documentation files (.md) unless a task explicitly calls for it
- Do not consolidate or merge tasks; each task must be completed as a discrete unit

## Constraints
- Tasks #64, #66, #65, and #68 address recurring M6 casualties and are the highest-priority items; they must not be deferred
- Task #56-2 is a sub-task of #56 and must not begin before #56 is complete or confirmed already satisfied
- Task #57 is a timezone fix with low difficulty (D:20) but also low confidence (C:45); investigate root cause before patching
- Task #58 has the lowest confidence score (C:40) in the set; a grep-based investigation of how pathfinder-mission-team parses TASKS.md sub-entries is required before any structural change
- All log-field changes (#60, #61, #62) must not break existing MISSION-LOG parsing used by other skill steps
- Permission-hook rules must not be loosened to work around compound-command denials; the commands themselves must be restructured

## Test Criteria
- MT-3d plan-rename block executes without permission-hook denial on a sample multi-file rename
- All subagent prompts pass a grep for "~/.claude" with zero matches
- Implementer prompt includes a mandatory grep step confirmed present in the updated SKILL.md
- MT-3d hard-block fires and re-queues a task when Status, Implementation, or Timestamp fields are missing
- MISSION-LOG-active.md is empty (or contains only a header) after MT-4 completes
- Premission rank is stored in PRIORITY_MAP and MT-2/MT-3a sort output matches the user-confirmed priority order
- PreCompact hook events appear in the numbered MISSION-LOG file, not only in MISSION-LOG-active.md
- DEFAULT-PERMISSIONS baseline is loaded at premission start and per-mission manifests extend it rather than overwrite it
- MISSION-LOG Task Status section contains Initial and Current Estimated Completion Time fields after MT-1 completes
- TASKS.md sub-entries (#N-2 format) are parsed correctly by mission-team without being mistaken for decomposed sub-tasks
