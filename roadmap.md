# Codex Suggestions

## Purpose
This file tracks improvement ideas, technical debt, reliability upgrades, security follow-ups, and other future work for this project.

It is not a commitment list. It is a place to keep useful ideas from getting lost.

## Tracker
- Next suggestion number: 21
- Rule: never reuse or renumber old suggestion IDs, even if an item is completed or removed later.
- Status values: `open`, `planned`, `in-progress`, `blocked`, `done`, `dropped`

## How To Use
Add items here when:
- something should be improved, but not as part of the current task
- a weakness is noticed in structure, reliability, security, or maintainability
- tests, documentation, or tooling are missing
- a recurring pain point suggests a better pattern

Keep each item short and practical:
- what the issue is
- why it matters
- a suggested next step

Format each item like:
- `1. [open] [Category] {Model Name}: Short title. Why it matters. Suggested next step.`

When adding a new item:
- place it in the highest-priority section that fits: `Now`, `Next`, `Later`, or `Icebox`
- include a category tag such as `[Reliability]`, `[Code Quality]`, `[Developer Experience]`, `[Documentation]`, `[Security]`, or `[Roadmap]`
- include the agent/model name at the start of the entry title in braces
- example: `11. [open] [Reliability] {GPT-5 Codex}: Lorem ipsum. Why it matters. Suggested next step.`

Authorship and updates:
- the first `{Model Name}` is the original author of the suggestion
- any agent may update status, progress notes, or wording without changing the original author tag
- if a different model substantially changes the meaning, scope, or recommendation, keep the original author tag and append `{updated by Model Name}`
- do not replace the original author tag unless the entry is being fully rewritten from scratch
- minor edits do not need an updater tag

When something is completed:
- keep the same number
- change only the status to `done`
- optionally add a short completion note instead of deleting the item

## Suggestions

Priority rule of thumb:
- `Now`: user-visible correctness, silent-data-risk fixes, and work already in progress
- `Next`: important follow-up work that reduces regression risk or closes known consistency gaps
- `Later`: valuable structural improvements that are safer after correctness and validation are stronger
- `Icebox`: worthwhile, but not currently justified against more urgent work

## Now
7. [in-progress] [Roadmap] {GPT-5 Codex}: Stabilization pass: fix note rendering to use the live section/field IDs from `data/sections.yml`, then add regression tests around the generated note text. ID-drift fixes applied (objective_section, remedial_section, infection_control_section, subjective_section). Root cause: group IDs and section IDs share namespace; note.rs was referencing group IDs in several places. Regression tests (suggestion #1) still open.
1. [open] [Reliability] {GPT-5 Codex}: Add golden-note tests that build a representative session and assert the rendered note includes each configured section and header field by current data IDs. This would catch ID drift like `subjective_section` vs `subjective` before it reaches users.
8. [planned] [Roadmap] {GPT-5 Codex}: Data consistency pass: migrate every remaining read/write path to the flat-block model, including user-added list entries.
9. [planned] [Roadmap] {GPT-5 Codex}: Validation pass: strengthen loader errors for ambiguous IDs and add a fast validation command or test that checks all data files together.
11. [open] [Roadmap] {Claude Sonnet 4.6}: Section-ID audit complete. All cfg.id comparisons in note.rs now match canonical section IDs in sections.yml. Pattern to watch: group IDs (subjective, objective, post_tx) look like section IDs but are not -- note.rs must always reference the child section ID (subjective_section, objective_section, post_treatment). Suggestion #1 (golden-note tests) would catch this automatically.
16. [planned] [Roadmap] {GPT-5 Codex}: Full hierarchy cutover: replace section-centric structure with typed `contains:` references, first-class collections, and structural note traversal. Why it matters: the current runtime still merges YAML globally but gives collections ambient access to all lists and still hard-codes note layout through section-centric assumptions. Suggested next step: implement the one-pass plan in `operations/plans/PLAN-74-full-cutover-typed-contains.md` and validate with real-data load tests plus golden-note coverage.

## Next
2. [done] [Reliability] {GPT-5 Codex}: Unify list-select persistence on one YAML format. The app now loads list-select options from hierarchy `lists:` files, and add-entry save/reload now reads and writes that same structure instead of the legacy `entries:` schema.
3. [open] [Reliability] {GPT-5 Codex}: Tighten flat-loader reference rules so child IDs are globally unique or type-qualified. The current validation allows the same `id` across block types even though reference resolution uses a plain `id -> index` map, which makes some child links ambiguous.
5. [open] [Developer Experience] {GPT-5 Codex}: Add task-focused integration tests for user-editable data flows, especially list-select "add entry" and note preview generation. The unit suite is healthy, but these cross-file behaviors are where the current regressions slipped through.
6. [open] [Documentation] {GPT-5 Codex}: Document the flat-data invariants in one short reference: allowed block IDs, whether IDs must be globally unique, and which files are still intentionally exempt from the flat format. That would make future YAML changes safer.
15. [open] [Roadmap] {GPT-5 Codex}: Define stable per-section editable-document anchors before completing the desktop rewrite. The tray app needs safe section-level replacement inside `editable_note`, but current note rendering groups multiple sections under shared headings. Suggested next step: make the anchor contract explicit in the rewrite docs and keep document helpers keyed to stable per-section anchors, not only top-level headings.
17. [open] [Security] {GPT-5 Codex}: Add explicit regression checks for clipboard-import privacy boundaries. The planned clinic-note import flow is intentionally memory-only, and accidental autosave, logging, or persistence would be a high-impact failure. Suggested next step: add tests and code-review checkpoints that verify imported patient note text is never written to config, data files, logs, or restored across restart.
18. [done] [Roadmap] {GPT-5 Codex}: Modal stream UI prototype: keep the current active modal interaction, but add read-only neighboring teaser cards so users can see adjacent modal states and collection previews simultaneously. Implemented in `src/modal.rs` and `src/ui.rs` per `operations/plans/IMPLEMENTATION-BRIEF-modal-stream-ui.md`, with automated coverage for simple-list teaser snapshots and collection preview neighbors.
19. [done] [Roadmap] {GPT-5 Codex}: Modal stream V2 planning and tuning pass: preserve the agreed next-step product direction and then implement it in phases. Why it matters: the prototype works, but the next iteration now has important product rules that are easy to lose in chat alone, including stub-card priority, animated slide transitions with tunable easing, future chunked/unit paging, and a top entry-composition panel with span-level override semantics. Completion note: all five planned phases are now implemented through `v0.3.8-alpha`, including stub packing, motion, the composition panel, field-level manual overrides, and chunked/unit modal paging.
20. [open] [Developer Experience] {GPT-5 Codex}: Stop tests from writing `config.yml` into the repo root. Why it matters: the current App tests can leave runtime artifacts in the worktree, which is noisy and increases the risk of accidentally staging local config. Suggested next step: route config persistence through a temp/test data dir in App tests or make config writes injectable/no-op under targeted test setups.

## Later
10. [planned] [Roadmap] {GPT-5 Codex}: Product flexibility pass: make note headings and boilerplate data-driven so changing the clinical template does not require Rust edits.
4. [open] [Code Quality] {GPT-5 Codex}: Move note layout metadata out of hard-coded Rust matches and into data. Section headings, note grouping, and header field IDs are duplicated in `note.rs`, which makes data migrations easy to break silently.
12. [open] [Code Quality] {GPT-5 Codex}: Namespace sticky-value keys by section and field for future multi_field reuse. Keys like `date.year` work for the current header, but they can collide once other multi_field sections reuse field or part IDs. Suggested next step: adopt a stable scheme like `<section_id>.<field_id>` and `<section_id>.<field_id>.<part_id>`, then plan a backward-compatible migration if existing persisted sticky values must be preserved.
13. [open] [Code Quality] {GPT-5 Codex}: Replace hard-coded layout mode strings with a typed config model before adding more modes. `config.rs` still treats layout choice as string literals, which is fragile if you plan to add more arrangements. Suggested next step: define a small enum or equivalent serialized config shape for layout modes, then update load/save paths to validate modes explicitly.
14. [open] [Code Quality] {GPT-5 Codex}: Make column layout and focus order data-driven so future invisible columns are possible. The app likely assumes the fixed map/wizard/preview set in multiple places, which will make hidden scrollable columns awkward to add safely. Suggested next step: audit rendering, sizing, focus movement, and key-handling assumptions about column count and visibility, then design a config model that separates column existence, visibility, and navigation order.

## Icebox
- None yet.

## Security
- None yet.
