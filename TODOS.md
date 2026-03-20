# TODOS

## scribblenot

### Returning-patient note continuation

**What:** When selected text exists at the time of Alt+T launch, auto-import it as a previous note and pre-populate sections with the prior content, ready for new entries to be appended under matching headings.

**Why:** Returning patients have a consistent treatment plan - copying the last note and appending only the changes (new symptoms, updated sleep/exercise, revised objective findings) is the current workflow. This would make that flow keyboard-native instead of manual copy-paste-edit.

**Context:** On Alt+T launch, the tray process can read the current selection via the clipboard (or Windows selection API). If the content looks like a structured clinical note (headings matching the known section taxonomy), parse it into pre-populated section state. New entries added in the session are then appended under the appropriate heading rather than starting from scratch. Key challenges: (1) detect selection vs. regular clipboard content reliably; (2) parse the note format back into section state (reverse of the renderer); (3) decide what "append under heading" means for block-select sections (Treatment) vs. free-text sections. Start with free-text sections only (ADL, Subjective, Objective) since those are the ones that actually change between visits.

**Effort:** L
**Priority:** P2
**Depends on:** Core TUI + renderer complete (v1 shipped)

---

### Section defaults

**What:** Add optional `default_text` or `default: skip` fields to `sections.yml` entries.

**Why:** After real usage, patterns emerge — sections always skipped, or always opened with the same boilerplate phrase. All text is user-authored in `sections.yml`. No AI, fully offline.

**Context:** If `default: skip` is set, the section auto-advances without a keypress. If `default_text` is set, the section opens pre-filled with that text (user can edit or confirm with `[t]`). The Infection Control section already uses a version of this idea (pre-checked). Build after v1 is shipped and real usage reveals which sections benefit.

**Effort:** S
**Priority:** P2
**Depends on:** v1 shipped

---

### ~~Configurable section order~~ — superseded

This item is superseded by the `sections.yml` design decision. Sections (including order, names, map labels, and types) are fully configurable via `sections.yml` from day one. No separate feature needed.
