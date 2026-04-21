# Codex Suggestions

## Purpose
This file tracks improvement ideas, technical debt, reliability upgrades, security follow-ups, and other future work for this project.

It is not a commitment list. It is a place to keep useful ideas from getting lost.

## Tracker
- Next suggestion number: 36
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
- do not place `[done]` items in `Now`, `Next`, `Later`, or `Icebox`; those sections are for unfinished work only
- include a category tag from the list below; pick the one that best describes the nature of the work:
  - `[Reliability]` — gaps where bad data, silent failures, or incorrect output can slip through
  - `[Architecture]` — how the system is designed and structured; type system decisions, data-driven patterns, component relationships
  - `[UI]` — rendering, layout, visual correctness, animation behavior
  - `[Features]` — user-facing functionality and product direction
  - `[Tooling]` — tests, error messages, and dev infrastructure
  - `[Documentation]` — written specs, references, and behavior contracts
  - `[Security]` — data protection and privacy boundaries
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
- change the status to `done`
- move the item to `## Done`
- optionally add a short completion note instead of deleting the item

Maintaining ## Now:
- `## Now` always holds exactly 3 recommended branch entries — not individual tasks
- Branch A anchors on the single highest-priority item from `## Next`; tasks that share the same code area or are thematically adjacent ride along
- Branch B anchors on the highest-priority remaining item that can safely run in parallel with A; if A touches too wide to safely add a second branch, note that instead of forcing one
- Branch C anchors on the highest-priority remaining item that can safely run in parallel with both A and B; note any higher-priority items that are blocked by code overlap with open branches
- each entry is a branch name (kebab-case), a one-line theme, and the task IDs it covers; B and C should note any parallel-safety caveats or blocked items
- update all 3 entries whenever priorities shift significantly or a branch is opened and work begins

## Suggestions

Priority rule of thumb:
- `Now`: always exactly 3 recommended branches (A, B, C); A anchors on the highest-priority item; B and C each anchor on the next highest item that can safely run in parallel with the branches above; note conflicts rather than force unsafe groupings
- `Next`: important follow-up work that reduces regression risk or closes known consistency gaps
- `Later`: valuable structural improvements that are safer after correctness and validation are stronger
- `Icebox`: worthwhile, but not currently justified against more urgent work
- `Done`: completed items only; keep them here so active sections stay readable

## Now
**A** `modal-correctness` — remaining confirmed-slot and modal correctness follow-through (#31, #27, #32, #35) — `#33` is deferred with an investigation note, and #23 still conflicts with this code area
**B** `schema-anchors` — stable document anchors and loader schema tightening; safe to open alongside A (#15, #3, #22)
**C** `explicit-hotkeys` — authored hint hotkeys and validation; safe to open alongside A and B (#29) — #23 remains blocked until A closes

## Next
31. [in-progress] [Reliability] [I: 70, D: 65] {GPT-5 Codex}: Scope item `assigns` per confirmed field slot instead of flattening them into one global map. Why it matters: the narrow stale-state fixes make current modal editing safer, but confirmed rendering still merges assigned format-list outputs across fields and repeats, so one slot can silently overwrite another. Suggested next step: implement `operations/plans/PLAN-assignment-slot-scoping-regression.md` (structural fix already landed; regression coverage is the remaining work). Note: `PLAN-assignment-slot-scoping.md` is superseded.
15. [in-progress] [Architecture] [I: 70, D: 55] {GPT-5 Codex}: Define stable per-section editable-document anchors before completing the desktop rewrite. The tray app needs safe section-level replacement inside `editable_note`, but current note rendering groups multiple sections under shared headings. Suggested next step: make the anchor contract explicit in the rewrite docs and keep document helpers keyed to stable per-section anchors, not only top-level headings. Plan: `operations/plans/section-anchor-contract.md`.
23. [open] [Architecture] [I: 65, D: 70] {GPT-5 Codex}: Stop flattening the authored runtime tree back into section-count math for navigation and preview grouping. Why it matters: `App` still derives group membership by walking `group.sections` lengths even though the loader already builds a typed runtime hierarchy, which keeps structural bugs alive whenever nodes stop behaving like plain sections. Suggested next step: key interactive state and navigation entries by runtime node ID, then derive the flat wizard/map slice from that tree instead of maintaining parallel structural views.
27. [in-progress] [UI] [I: 60, D: 50] {GPT-5 Codex}: Modal stub preview-sequence fix for repeat-joiner lists. Why it matters: the current modal stream can show a terminal green `+` on non-terminal states like `obmuscle_field`'s `muscle` list because teaser generation stops early for repeat-joiner lists even when real field flow continues. Suggested next step: implement `operations/plans/PLAN-modal-stub-option-1-preview-sequence-fix.md` and add regression coverage for repeat lists with downstream modal states.
28. [open] [Architecture] [I: 55, D: 65] {GPT-5 Codex}: Establish semantic stub authority for modal edges. Why it matters: stub meaning is currently inferred from preview-sequence availability rather than real modal behavior, which will keep causing correctness drift as repeat, nested, branch, and other modal flows get more complex. Suggested next step: implement `operations/plans/PLAN-modal-stub-option-2-semantic-authority.md` so `>`, `+`, and `-` come from explicit edge semantics instead of teaser availability alone.
22. [open] [Architecture] [I: 55, D: 60] {GPT-5 Codex}: Replace inferred string `section_type` with an explicit authored `body` enum for sections. Why it matters: the loader currently infers runtime behavior from child shape and still dispatches via string matches, which is exactly the kind of ambiguity that causes one-off fixes when new section forms appear. Suggested next step: add `body:` to the authored schema, migrate real data, and make runtime dispatch use a typed enum instead of `"multi_field"` / `"list_select"` / `"free_text"` strings.
3. [open] [Reliability] [I: 55, D: 45] {GPT-5 Codex}: Tighten flat-loader reference rules so child IDs are globally unique or type-qualified. The current validation allows the same `id` across block types even though reference resolution uses a plain `id -> index` map, which makes some child links ambiguous.
32. [in-progress] [UI] [I: 55, D: 30] {GPT-5 Codex}: Detail the Entry Composition box lifecycle for modal open, exit, and confirm transitions. Why it matters: the modal stack now has lifecycle motion, but the top composition panel still needs explicit product rules for whether it fades, slides with the active unit, pins independently, or collapses on its own timing. Suggested next step: write a short behavior spec for open/exit/confirm, then add focused regressions for panel retention, spacing, and any chosen motion semantics.
29. [planned] [Features] [I: 50, D: 55] {GPT-5 Codex}: Add explicit single-character `hotkey:` support for hinted entities so authored sections, header fields, and list items can reserve mnemonic labels instead of relying only on generated hints. Why it matters: the current hint system is fully generated, which makes high-frequency actions harder to memorize and blocks stable mnemonic workflows. Suggested next step: implement `operations/plans/PLAN-explicit-hotkeys-for-sections-fields-items.md`, with validation for duplicate local hotkeys, command-binding collisions, shorthand-item limitations, and text-entry precedence.
33. [open] [UI] [I: 50, D: 60] {GPT-5 Codex}: Investigate active modal row border width mismatch versus inactive modal and search-bar width. Why it matters: the active modal's confirmed-row border still renders narrower than the inactive modal rows, which causes visible jitter and inconsistent alignment during navigation. Suggested next step: continue auditing the Iced width and child-layout chain around the active row container and button path in `src/ui/mod.rs`, then capture a focused visual regression check or screenshot fixture once the layout model is understood. Status note: deferred from `modal-correctness` after multiple focused attempts showed no visible change. Investigation note: `operations/plans/REPORT-33-modal-row-width-investigation.md`.
35. [in-progress] [UI] [I: 50, D: 35] {GPT-5 Codex}: Show the confirm border-box style on the item before confirm-triggered transitions begin in the final modal. Why it matters: confirming an item in either final-modal path currently starts the unit-transition or confirm-close motion immediately, so the row never visibly enters its confirmed styling during the transition and ends up looking too similar to a plain `nav_right` focus move where no confirmation happened. Suggested next step: make confirm actions apply the confirmed-row visual state for at least the first rendered transition frame, then add a focused regression or screenshot check that distinguishes confirm from `nav_right` in both final-modal paths.
21. [open] [Architecture] [I: 40, D: 35] {GPT-5 Codex}: Avoid redundant modal-unit recalculation on no-op data refreshes. Why it matters: the new unit layout is intentionally precomputed on open, refresh, and resize, but repeated YAML reloads with unchanged content still pay the full derivation cost. Suggested next step: cache a fast data checksum (for example CRC32) and skip modal-unit rebuild work when the loaded content hash is unchanged.
6. [open] [Documentation] [I: 40, D: 20] {GPT-5 Codex}: Document the flat-data invariants in one short reference: allowed block IDs, whether IDs must be globally unique, and which files are still intentionally exempt from the flat format. That would make future YAML changes safer.
8. [open] [Architecture] [I: 35, D: 30] {GPT-5 Codex}: Remove legacy `data_file` compatibility from list/checklist runtime paths now that in-app custom list-entry persistence is gone. Why it matters: hierarchy loading already builds section-owned list/checklist data, but `App` still carries fallback reads keyed by `cfg.data_file`, which keeps obsolete dual-path behavior alive and makes the current model harder to reason about. Suggested next step: delete the remaining list/checklist `data_file` fallback branches, update affected tests/fixtures, and document that mutable list authoring now belongs in YAML or note-level composition flows rather than in-app list persistence.

## Later
34. [open] [Architecture] [I: 25, D: 35] {Claude Sonnet 4.6}: Consider renaming `label` to `nav_label` on field blocks for schema consistency with sections and groups. Why it matters: sections and groups already distinguish `label` (wizard title) from `nav_label` (map label), but field blocks use only `label` for both purposes, which is inconsistent. Before renaming, determine whether field `label` currently serves any distinct role beyond the wizard prompt - if it does, a two-key split may be needed instead of a simple rename. Suggested next step: audit all field `label` usages in the Rust loader and UI to confirm whether a single `nav_label` key would cover all current uses, then plan a data migration if it does.
4. [open] [Architecture] [I: 50, D: 55] {GPT-5 Codex}: Move note layout metadata out of hard-coded Rust matches and into data. Section headings, note grouping, and header field IDs are duplicated in `note.rs`, which makes data migrations easy to break silently.
10. [planned] [Features] [I: 45, D: 60] {GPT-5 Codex}: Product flexibility pass: make note headings and boilerplate data-driven so changing the clinical template does not require Rust edits.
12. [open] [Reliability] [I: 45, D: 35] {GPT-5 Codex}: Namespace sticky-value keys by section and field for future multi_field reuse. Keys like `date.year` work for the current header, but they can collide once other multi_field sections reuse field or part IDs. Suggested next step: adopt a stable scheme like `<section_id>.<field_id>` and `<section_id>.<field_id>.<part_id>`, then plan a backward-compatible migration if existing persisted sticky values must be preserved.
24. [open] [Tooling] [I: 40, D: 60] {GPT-5 Codex}: Preserve source file and line provenance for hierarchy IDs and child refs so validation errors can point to exact authored locations. Why it matters: current validation can explain what is wrong and how to fix it, but merged semantic errors still lose file/line context after deserialization, which slows down debugging in larger YAML edits. Suggested next step: carry source spans for top-level nodes and `contains` refs through parse/merge, then include `path:line` in missing-ref, wrong-kind, and duplicate-id errors.
13. [open] [Architecture] [I: 40, D: 30] {GPT-5 Codex}: Replace hard-coded layout mode strings with a typed config model before adding more modes. `config.rs` still treats layout choice as string literals, which is fragile if you plan to add more arrangements. Suggested next step: define a small enum or equivalent serialized config shape for layout modes, then update load/save paths to validate modes explicitly.
14. [open] [Architecture] [I: 35, D: 55] {GPT-5 Codex}: Make column layout and focus order data-driven so future invisible columns are possible. The app likely assumes the fixed map/wizard/preview set in multiple places, which will make hidden scrollable columns awkward to add safely. Suggested next step: audit rendering, sizing, focus movement, and key-handling assumptions about column count and visibility, then design a config model that separates column existence, visibility, and navigation order.

## Icebox
26. [open] [Architecture] [I: 20, D: 30] {Claude Sonnet 4.6}: Consider merging ModalArrivalLayer and ModalDepartureLayer into a single struct. Why it matters: both layers are currently created at the same instant with the same timing settings - the separation exists only to support a planned Part 3 feature where the departure runs on independent timing. If that feature is deferred, a merged struct would reduce complexity. Suggested next step: get a functional baseline first, then evaluate whether Part 3 independent timing is still on the roadmap before committing to the split.
25. [open] [Architecture] [I: 15, D: 45] {Claude Sonnet 4.6}: Consider cancelling in-flight transitions on window resize rather than relying on frozen geometry. Why it matters: UnitGeometry and UnitContentSnapshot exist primarily to insulate the animation from mid-flight layout changes - if a resize simply cancelled and restarted the transition from the new layout, much of the freeze machinery could be dropped or simplified. Suggested next step: evaluate whether cancel-and-restart on resize is perceptually acceptable (quick resize gestures during animations are rare), then assess how much of the freeze scaffolding could be removed.

## Done
1. [done] [Reliability] {GPT-5 Codex}: Add golden-note tests that build a representative session and assert the rendered note includes each configured section and header field by current data IDs. Completion note: the regression suite in `src/note.rs` now exercises real-data group ordering, current-ID field seeding, and a representative golden-note fixture without OS-specific newline failures.
2. [done] [Reliability] {GPT-5 Codex}: Unify list-select persistence on one YAML format. Historical note: this work was later superseded for product direction because reusable list-item authoring was removed from the UI; current editing now belongs in YAML authoring or per-note composition/manual override flows instead of in-app list persistence.
5. [done] [Tooling] [I: 60, D: 50] {GPT-5 Codex}: Add task-focused integration tests for current user-editable flows, especially composition/manual-override note preview generation and editable-document sync. Completion note: `src/app.rs` now covers manual-override export, Ctrl+R override reset, free-text confirm via the real key-handler path, and multi-section editable-note sync retention.
7. [done] [Roadmap] {GPT-5 Codex}: Stabilization pass: fix note rendering to use the live section/field IDs from `data/sections.yml`, then add regression tests around the generated note text. ID-drift fixes applied (objective_section, remedial_section, infection_control_section, subjective_section). Root cause: group IDs and section IDs share namespace; note.rs was referencing group IDs in several places. Completion note: `src/note.rs` now covers authored group order, managed section heading placement, live multi_field outputs keyed by current section IDs, and a representative golden-note render with line-ending-safe comparison.
9. [done] [Roadmap] {GPT-5 Codex}: Validation pass: strengthen loader errors for ambiguous IDs and add a fast validation command or test that checks all data files together. Completion note: duplicate IDs across hierarchy kinds were already rejected by the typed registry; this pass tightened the human-facing error text, added `cargo run -- --validate-data` for fast hierarchy plus `keybindings.yml` validation, and covered the path with real-data and invalid-keybindings tests.
11. [done] [Roadmap] {Claude Sonnet 4.6}: Section-ID audit complete. All cfg.id comparisons in note.rs now match canonical section IDs in sections.yml. Pattern to watch: group IDs (subjective, objective, post_tx) look like section IDs but are not -- note.rs must always reference the child section ID (subjective_section, objective_section, post_treatment). Completion note: the note-render regression suite in `src/note.rs` now exercises those canonical IDs through real-data and golden-note coverage.
16. [done] [Roadmap] {GPT-5 Codex}: Full hierarchy cutover: replace the remaining section-centric runtime with authored-tree navigation, rendering, and document sync on top of the already-landed typed `contains:` and first-class collection loader work. Completion note: runtime-tree traversal now drives note rendering and editable-document generation, app navigation grouping now derives from `RuntimeTemplate` rather than `SectionGroup.sections` math, and focused regressions cover runtime navigation order plus template-driven note ordering.
17. [done] [Security] [I: 75, D: 40] {GPT-5 Codex}: Add explicit regression checks for clipboard-import config-persistence boundaries. Completion note: `src/app.rs` now uses temp-dir-backed save-path regressions to assert `editable_note` never lands in persisted config, and `src/import.rs` now exists with a stub `try_parse_clipboard_note` plus non-panicking baseline tests for Phase 2.
18. [done] [Roadmap] {GPT-5 Codex}: Modal stream UI prototype: keep the current active modal interaction, but add read-only neighboring teaser cards so users can see adjacent modal states and collection previews simultaneously. Implemented in `src/modal.rs` and `src/ui.rs` per `operations/plans/IMPLEMENTATION-BRIEF-modal-stream-ui.md`, with automated coverage for simple-list teaser snapshots and collection preview neighbors.
19. [done] [Roadmap] {GPT-5 Codex}: Modal stream V2 planning and tuning pass: preserve the agreed next-step product direction and then implement it in phases. Why it matters: the prototype works, but the next iteration now has important product rules that are easy to lose in chat alone, including stub-card priority, animated slide transitions with tunable easing, future chunked/unit paging, and a top entry-composition panel with span-level override semantics. Completion note: all five planned phases are now implemented through `v0.3.8-alpha`, including stub packing, motion, the composition panel, field-level manual overrides, and chunked/unit modal paging.
20. [done] [Tooling] [I: 45, D: 25] {GPT-5 Codex}: Stop tests from writing `config.yml` into the repo root. Completion note: `Config::save()` now treats an empty `data_dir` as a no-op, targeted config tests cover both the empty-path and normal-save cases, and the full suite leaves the existing root-level ignored `config.yml` untouched.
30. [done] [Roadmap] {GPT-5 Codex}: Add modal lifecycle transitions for open, exit, and confirm so entering and leaving the wizard matches the existing unit-to-unit motion language. Completion note: `src/transition.rs`, `src/app.rs`, and `src/ui/mod.rs` now model explicit one-sided `ModalOpen` and `ModalClose` lifecycle layers, semantic exit/confirm helpers preserve close animations after `self.modal` is cleared, and focused regressions cover lifecycle direction, unit-owned stub fading, overlay retention, and non-simple close fallback.

## Security
- None yet.
