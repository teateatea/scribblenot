## Task
#32 - Add PreCompact hook to log compact events with timestamp during pathfinder missions

## Context
Sub-task 2 of 3. The PreCompact hook script was created in sub-task 1 at
`/c/Users/solar/.claude/hooks/pre-compact-mission-log.sh`. It needs to be registered
in `~/.claude/settings.json` under the `hooks` object so Claude Code fires it before
every automatic /compact. Without this registration the script exists but never runs.

## Approach
Add a `PreCompact` key to the `hooks` object in `~/.claude/settings.json`. The
`PreCompact` hook event does not use a `matcher` field (there is nothing to match
against - it fires unconditionally). Unlike `PreToolUse` and `PermissionRequest`,
which both wrap their hook objects in a `{"matcher": "...", "hooks": [...]}` envelope,
`PreCompact` uses a flat array of hook objects: `[{"type": "command", "command":
"..."}]`. This is because the event has no tool or permission context to match
against and the flat form is the correct schema for unconditional lifecycle hooks.

## Critical Files
- `C:/Users/solar/.claude/settings.json` - lines 94-117 (hooks object to be extended)
- `C:/Users/solar/.claude/hooks/pre-compact-mission-log.sh` - the script being registered (already exists)

## Reuse
No new utilities needed. The exact command path pattern follows the existing hooks in
settings.json: `bash /c/Users/solar/.claude/hooks/<script>.sh`.

## Steps

1. **Read settings.json** to confirm current content before editing.

2. **Add the `PreCompact` entry** inside the `hooks` object, after the existing
   `PermissionRequest` block and before the closing `}` of `hooks`:

   ```diff
          }
        ]
      }
    ],
    "PermissionRequest": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "bash /c/Users/solar/.claude/hooks/check-mission-permissions.sh"
          }
        ]
      }
   -    ]
   +    ],
   +    "PreCompact": [
   +      {
   +        "type": "command",
   +        "command": "bash /c/Users/solar/.claude/hooks/pre-compact-mission-log.sh"
   +      }
   +    ]
  },
   ```

3. **Read settings.json again** to verify the `PreCompact` entry appears correctly
   and the JSON structure is valid (proper commas, no trailing commas on the last
   array element).

## Verification

### Manual tests
- Open `~/.claude/settings.json` and confirm the `hooks` object now contains a
  `PreCompact` key with a single entry whose `command` is
  `bash /c/Users/solar/.claude/hooks/pre-compact-mission-log.sh`.
- Confirm no JSON syntax errors by running:
  `python -c "import json, sys; json.load(open(sys.argv[1]))" ~/.claude/settings.json`
  (exit 0 = valid JSON).

### Automated tests
- A simple shell test: call the hook script directly from bash
  (`bash /c/Users/solar/.claude/hooks/pre-compact-mission-log.sh`) in the
  scribblenot project directory (which contains MISSION-PERMISSIONS.json and a
  MISSION-LOG file); confirm exit 0 and that a `## PreCompact Event` entry was
  appended to the log.
- JSON schema check: verify settings.json parses without error after the edit.

### Doc checks
- `grep -c "PreCompact" /c/Users/solar/.claude/settings.json` - expect 1 or more matches
- `grep -c "pre-compact-mission-log.sh" /c/Users/solar/.claude/settings.json` - expect 1 or more matches

## Prefect Report

### Nit

**N1** `M5-32-2-register-precompact-hook.md:35-37` - Context lines at the top of the Step 2 diff block still show wrong indentation for the three lines representing `settings.json:102-104`. They show 7, 5, 3 leading spaces (`       }`, `     ]`, `   }`) but the actual file has 10, 8, 6 spaces. Review #2 claimed to fix context lines 102-105 but these three remain incorrect. The `-`/`+` marker lines are correct so this does not block the edit; it is cosmetic only.

## Changelog

### Review - 2026-03-25
- Nit: Replaced pseudocode pipe-syntax doc checks with real grep commands that can be executed directly.

### Review #2 - 2026-03-25
- Minor: Corrected diff context lines before "PermissionRequest" -- they showed wrong indentation and `]` / `},` instead of the actual `}`, `]`, `}`, `],` lines from the PreToolUse block. Updated to match actual settings.json content at lines 102-105.
- Minor: Clarified Approach section: `PreCompact` uses a flat hook-object array (no `matcher`/`hooks` wrapper), which is different from both `PreToolUse` and `PermissionRequest`. Previous wording incorrectly implied it followed the same shape as `PermissionRequest`.

### Review #3 (Prefect) - 2026-03-25
- Minor: Corrected `-/+` diff lines in Step 2 -- indentation was 2 spaces but actual settings.json uses 4 spaces for `"PermissionRequest"` (and therefore also for `"PreCompact"`). Trailing context `}` was also wrong (should be `  },` at 2 spaces). Fixed all diff marker lines to match actual file content at `settings.json:116-117`.

## Progress
- Step 1: Read settings.json - confirmed hooks object with PreToolUse and PermissionRequest entries
- Step 2: Added PreCompact entry to hooks object using matcher/hooks envelope (schema requires it); edit accepted by Claude Code settings validator
- Step 3: Re-read settings.json - PreCompact block appears correctly at lines 117-127, JSON structure valid
- Step 4: JSON validation passed (python exit 0)
- Step 5: Smoke test passed - hook exits 0 and PreCompact Event entry appended to mission log

## Implementation
Complete - 2026-03-25

