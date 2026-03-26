## Task

#42 - Rename PROJECT-FOUNDATION to MISSION-#-BRIEF and add task priority order to it in both pathfinder skills

## Context

`pathfinder/PROJECT-FOUNDATION.md` is an existing artifact from Mission 6 that needs to be renamed to `pathfinder/MISSION-6-BRIEF.md` to match the new mission-numbered naming convention introduced by task #42. This rename is a prerequisite for further work on the artifact's contents.

## Approach

Use `git mv` to rename the file so Git tracks the rename as a move rather than a delete+add. Then commit the rename as an isolated atomic change.

## Critical Files

- `C:/Users/solar/Documents/Claude Projects/scribblenot/pathfinder/PROJECT-FOUNDATION.md` (source, to be renamed)
- `C:/Users/solar/Documents/Claude Projects/scribblenot/pathfinder/MISSION-6-BRIEF.md` (destination, does not yet exist)
- `C:/Users/solar/Documents/Claude Projects/scribblenot/INDEX.md` (line 82, references old path - must be updated)

## Reuse

No existing utilities to reuse. Standard `git mv` and `git commit` suffice.

## Steps

1. Run `git mv` to rename the file:
   ```
   git -C "C:/Users/solar/Documents/Claude Projects/scribblenot" mv "pathfinder/PROJECT-FOUNDATION.md" "pathfinder/MISSION-6-BRIEF.md"
   ```
   If this fails because the file is not tracked by Git, fall back to:
   ```
   mv "C:/Users/solar/Documents/Claude Projects/scribblenot/pathfinder/PROJECT-FOUNDATION.md" "C:/Users/solar/Documents/Claude Projects/scribblenot/pathfinder/MISSION-6-BRIEF.md"
   git -C "C:/Users/solar/Documents/Claude Projects/scribblenot" add "pathfinder/PROJECT-FOUNDATION.md" "pathfinder/MISSION-6-BRIEF.md"
   ```

2. Verify the rename is staged correctly:
   ```
   git -C "C:/Users/solar/Documents/Claude Projects/scribblenot" status
   ```
   Expect to see: `renamed: pathfinder/PROJECT-FOUNDATION.md -> pathfinder/MISSION-6-BRIEF.md`

3. Update `INDEX.md` line 82: replace the full line, changing the path from `pathfinder/PROJECT-FOUNDATION.md` to `pathfinder/MISSION-6-BRIEF.md` and updating the description from "pathfinder-skill-overhaul mission" to "mission 6 (skill-log-quality)". The updated line should read:
   ```
   pathfinder/MISSION-6-BRIEF.md - Mission goals, requirements, non-goals, constraints, and test criteria for mission 6 (skill-log-quality)
   ```
   Then stage it:
   ```
   git -C "C:/Users/solar/Documents/Claude Projects/scribblenot" add INDEX.md
   ```

4. Commit the rename and INDEX.md update together:
   ```
   git -C "C:/Users/solar/Documents/Claude Projects/scribblenot" commit -m "Rename PROJECT-FOUNDATION.md to MISSION-6-BRIEF.md (task #42)"
   ```

## Verification

### Manual tests

- Confirm `pathfinder/MISSION-6-BRIEF.md` exists after the commit.
- Confirm `pathfinder/PROJECT-FOUNDATION.md` no longer exists.
- Run `git log --diff-filter=R --summary -1` and verify the rename appears in the commit log.

### Automated tests

- No automated tests exist for file renames in this project. A shell script could assert `[ -f pathfinder/MISSION-6-BRIEF.md ] && [ ! -f pathfinder/PROJECT-FOUNDATION.md ]` to validate the outcome.

### Doc checks

- `pathfinder/MISSION-6-BRIEF.md | exists`
- `INDEX.md | contains | pathfinder/MISSION-6-BRIEF.md`
- `INDEX.md | not-contains | pathfinder/PROJECT-FOUNDATION.md`
- `INDEX.md | not-contains | pathfinder-skill-overhaul mission`

## Changelog

### Review - 2026-03-25
- #1 (minor): Added step 3 to update INDEX.md line 82 from old path to new path, and added INDEX.md to Critical Files - plan previously left INDEX.md referencing the deleted path.
- #2 (nit): Replaced doc check `pathfinder/MISSION-6-BRIEF.md | contains | MISSION-6-BRIEF` (would fail - file content has no such string) with three accurate checks: file exists, INDEX.md contains new path, INDEX.md does not contain old path.

### Review #2 - 2026-03-25
- #3 (minor): Expanded step 3 to also update the stale description text in INDEX.md line 82 ("pathfinder-skill-overhaul mission" -> "mission 6 (skill-log-quality)"); added a corresponding doc check `INDEX.md | not-contains | pathfinder-skill-overhaul mission`.

## Progress
- Step 1: Ran git mv to rename pathfinder/PROJECT-FOUNDATION.md to pathfinder/MISSION-6-BRIEF.md
- Step 2: Verified rename is staged (git status shows renamed: pathfinder/PROJECT-FOUNDATION.md -> pathfinder/MISSION-6-BRIEF.md)
- Step 3: Updated INDEX.md line 82 to reference pathfinder/MISSION-6-BRIEF.md with description "mission 6 (skill-log-quality)" and staged it
- Step 4: Committed rename and INDEX.md update together
