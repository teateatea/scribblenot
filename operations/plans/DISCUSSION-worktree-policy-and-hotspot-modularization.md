# Discussion: Worktree Policy and Hotspot Modularization

**Date:** 2026-04-24
**Status:** Recommended

## Plain-English Summary
Parallel worktrees are still worth using in this repo.

The real problem is not Git worktrees. The problem is that a small set of very large files acts as shared turf, so two otherwise separate branches can quietly overlap and then feel messy at merge time.

The safest default for Scribblenot is:

- keep using parallel worktrees
- only run sibling branches when they have clearly different ownership
- use stacked branches when one change builds on another
- sequence work when two branches need the same hotspot file for different reasons

That gives you speed where the work is genuinely separate without pretending that every feature is parallel-safe.

## Current Hotspots
As of 2026-04-24, the biggest merge-magnet source files are:

- `src/app.rs` - 6539 lines
- `src/data.rs` - 5076 lines
- `src/modal.rs` - 4959 lines
- `src/ui/mod.rs` - 4665 lines

Important authored-data hotspots:

- `data/treatment.yml` - 1000 lines
- `data/sections.yml` - 728 lines
- `data/subjective.yml` - 631 lines
- `data/objective.yml` - 397 lines

Important shared-doc hotspots:

- `roadmap.md`
- `README.md`
- `operations/MISSION-LOG-active.md`

These are the files most likely to create "invisible toe-stepping" even when the user-facing goals sound different.

## Two Options

### Option 1: Mostly Sequential Work
Use one active feature branch at a time unless the next task is obviously isolated.

**Pros**
- simplest mental model
- lowest merge stress
- easiest way to preserve context in a fast-moving codebase

**Cons**
- slower overall
- underuses worktrees
- encourages broad branches because "everything can wait for this one"

### Option 2: Disciplined Parallel Work
Run parallel worktrees only when each branch has a narrow owned surface, and stack branches when one depends on another.

**Pros**
- keeps momentum high
- gets real value from worktrees
- exposes weak ownership boundaries clearly

**Cons**
- requires more branch discipline
- requires earlier merges and more frequent resync
- still breaks down if hotspot files stay broad

## Recommendation
Choose **Option 2**.

For this repo, full sequencing is too conservative, but unconstrained parallel work is too optimistic. The right middle ground is disciplined parallel work with explicit ownership and a plan to reduce the hotspot files over time.

## Recommended Worktree Policy

### 1. Decide sibling vs stacked branches before opening the worktree
Use **sibling branches off `main`** when:

- the branches own different Rust modules
- the branches own different authored-data files
- neither branch depends on the other to make sense

Use **stacked branches** when:

- branch B builds on branch A's internal scaffolding
- both branches touch the same subsystem
- splitting them into siblings would duplicate or conflict on the same code movement

Sequence work instead of parallelizing when:

- both branches need `src/app.rs`, `src/data.rs`, `src/modal.rs`, or `src/ui/mod.rs`
- both branches need the same `.yml` file under `data/`
- both branches change the same user flow from different angles

### 2. Give each branch an ownership sentence
Before starting a branch, write one plain sentence in the branch notes, commit message draft, or mission doc:

`This branch owns <areas/files>. It may touch <shared seams>. It should not edit <other seams>.`

Example:

`This branch owns loader validation and source reporting in src/data.rs. It may touch diagnostics wiring. It should not edit modal or UI rendering code.`

If that sentence is hard to write, the branch is probably too broad.

### 3. Treat hotspot files as explicit shared turf
When a branch must touch one of these files:

- `src/app.rs`
- `src/data.rs`
- `src/modal.rs`
- `src/ui/mod.rs`
- `data/sections.yml`
- `data/treatment.yml`
- `roadmap.md`

assume another branch may also need it soon.

That means:

- keep the edit as small as possible
- merge sooner than usual
- resync other active branches immediately after merge

### 4. Keep branches short-lived
For Scribblenot, hidden merge pain rises quickly after a branch sits for more than a day or two while `main` keeps moving.

Practical rule:

- if a branch is not mergeable yet, but its foundation is stable, split and merge the foundation first
- if a branch stays open more than two days, resync it from `main`

### 5. Keep docs and roadmap changes out of unrelated branches when possible
`roadmap.md`, `README.md`, and mission logs are useful, but they create artificial conflicts.

Default rule:

- only change them when the branch is actually about that behavior or follow-up
- otherwise, save doc-only housekeeping for the merge branch or a short follow-up branch

### 6. Use a merge checklist
Before merging any active worktree branch:

1. Check whether `main` moved since the branch started.
2. Merge or rebase `main` into the branch.
3. Resolve hotspot-file conflicts immediately, not later.
4. Re-run the branch's focused validation.
5. Merge the branch as soon as it is coherent.

### 7. Use conflict repetition as architecture feedback
If the same file causes conflicts across two or three separate branches, stop treating that as "just Git being annoying."

That is a design signal that the file owns too many responsibilities.

## Practical Branch Rules for This Repo

Good candidates for sibling branches:

- diagnostics work vs note formatting work
- UI theme work vs loader validation work
- authored-data work in different `.yml` files when there is no shared schema shift

Good candidates for stacked branches:

- modal infrastructure, then modal behavior polish
- loader provenance, then higher-level diagnostic rendering
- collection-runtime groundwork, then collection UI refinement

Poor candidates for parallel sibling branches:

- two modal behavior changes that both need `src/modal.rs` and `src/app.rs`
- two schema changes that both need `src/data.rs` and `data/sections.yml`
- one branch refactoring a hotspot while another branch adds features inside the same hotspot

## Modularization Goal
Do **not** do a broad rewrite just to make worktrees feel cleaner.

Do targeted extractions that create real ownership boundaries and can land as no-behavior-change refactors.

The goal is not "small files." The goal is "branches can own different files for different reasons."

## Naming Strategy

### Option 1: Nested module directories with more `mod.rs` files
Examples:

- `src/data/mod.rs`
- `src/data/model.rs`
- `src/data/load.rs`

**Pros**
- conventional Rust layout
- simple module declarations
- good fit when a subsystem is expected to keep growing

**Cons**
- many editor tabs become `mod.rs`
- easy to open the wrong facade file by accident
- larger rename from the current single-file hotspots

### Option 2: Keep the public facade file and extract uniquely named siblings
Examples:

- `src/data.rs`
- `src/data_model.rs`
- `src/data_load.rs`
- `src/data_source.rs`

**Pros**
- explicit filenames in tabs and search results
- lower confusion during parallel work
- smaller first refactor because the public module entrypoint stays put

**Cons**
- slightly less idiomatic than directory-first layout
- may use `#[path = "..."]` or crate-level helper modules to keep names tidy

## Naming Recommendation
Choose **Option 2** for the first hotspot splits in this repo.

The main problem being solved is merge friction and navigation confusion, not textbook module aesthetics. Explicit filenames will be easier to work with while the codebase is still actively being reshaped.

### Naming rule for future hotspot splits
When splitting a large top-level source file, prefer this pattern:

- keep the existing public facade file when it already exists and callers already use it
- add helper files with the subsystem prefix in the filename
- avoid introducing a new directory-level `mod.rs` unless the subsystem is already a directory module or genuinely needs many nested internal submodules

Examples:

- keep `src/data.rs`, add `src/data_model.rs`, `src/data_load.rs`
- keep `src/modal.rs`, add `src/modal_flow.rs`, `src/modal_assignments.rs`
- keep `src/app.rs`, add `src/app_input.rs`, `src/app_navigation.rs`

This is not a universal Rust rule. It is a repo-local readability rule meant to keep editor tabs, ripgrep output, and merge views unambiguous during the modularization phase.

## Recommended Modularization Plan

### Phase 1: Split `src/data.rs` first
This file already contains several distinct jobs:

- authored schema types
- hierarchy loading and YAML document parsing
- source-index and provenance mapping
- validation and author-facing error reporting
- runtime hierarchy construction
- hint-label generation

Recommended target shape:

- `src/data.rs` - public exports and orchestration facade
- `src/data_model.rs` - authored and runtime structs/enums
- `src/data_load.rs` - directory loading, YAML document splitting, file merge rules
- `src/data_source.rs` - source anchors, source index, provenance helpers
- `src/data_validate.rs` - hierarchy validation and fix-hint reporting
- `src/data_runtime.rs` - runtime tree construction and flattening helpers
- `src/data_hints.rs` - hint generation and assignment logic

Why first:

- it is large
- its responsibilities are already distinct
- many future branches will want only one of those responsibilities

### Phase 2: Split `src/ui/mod.rs`
`src/ui/mod.rs` is large, but its seams are visible already.

Recommended target shape:

- `src/ui/mod.rs` - top-level `view()` wiring and exports
- `src/ui/style.rs` - shared colors, button/container styles, text helpers
- `src/ui/panes.rs` - map, wizard, editor, preview, status bar
- `src/ui/modal_overlay.rs` - simple modal, collection modal, composition panel, sizing helpers
- `src/ui/error_modal.rs` - error modal rendering and YAML fix-snippet formatting

Why second:

- it is a major merge magnet
- it already has one successful split (`src/ui/modal_unit.rs`)
- rendering work often overlaps only because everything still lives in one file

### Phase 3: Split authored data across more top-level `.yml` files
The loader already reads all top-level `*.yml` files in `data/` and merges them, while enforcing exactly one template across files.

That means data modularization can happen without changing the loader, as long as the files stay in the top-level `data/` directory and exactly one file owns the template.

Suggested direction:

- keep one template-owning file
- split large authored domains into separate top-level files by clinical area or ownership
- avoid splitting one concept across many tiny files too early

Reasonable first cuts:

- move treatment-specific groups/sections/fields/lists into multiple treatment-focused files
- keep subjective/objective domains separate if they already have natural ownership
- keep shared boilerplate or cross-cutting schema examples in their own stable file

### Phase 4: Split `src/modal.rs`
`src/modal.rs` mixes state shape, list flow, collection helpers, assignment logic, formatting, and template display resolution.

Recommended target shape:

- `src/modal/mod.rs` - exports and high-level orchestration
- `src/modal/types.rs` - modal structs/enums and lightweight helpers
- `src/modal/flow.rs` - `SearchModal` navigation and confirmation behavior
- `src/modal/assignments.rs` - item assigns and lookup resolution
- `src/modal/collections.rs` - collection preview/state helpers
- `src/modal/format.rs` - display template and repeat/join formatting helpers

Why not first:

- the file is central to behavior
- flow and state are tightly coupled
- it is worth doing after the cleaner `data` and `ui` extractions establish a pattern

### Phase 5: Split `src/app.rs` last and in slices
`src/app.rs` is the biggest file, but it is also the orchestration center. That makes it high-value and high-risk.

Recommended target shape:

- `src/app/mod.rs` - `App` type, exports, top-level orchestration
- `src/app/input.rs` - key parsing, binding matching, input helpers
- `src/app/state.rs` - enums/structs for focus, section state, flashes, composition spans
- `src/app/navigation.rs` - map/wizard focus movement and cursor logic
- `src/app/modal_actions.rs` - modal open/confirm/back/collection actions
- `src/app/reload.rs` - data/theme/config reload handling
- `src/app/composition.rs` - field span, preview, and substitution helpers

Why last:

- many branches touch app orchestration indirectly
- this split is easiest to get wrong if attempted broadly
- it becomes safer once `data`, `ui`, and `modal` expose cleaner module boundaries

## Suggested Order of Real Work
If the goal is to reduce merge pressure without opening a risky rewrite branch, use this order:

1. `refactor/data-module-split`
2. `refactor/ui-render-split`
3. `refactor/data-file-split`
4. `refactor/modal-module-split`
5. `refactor/app-module-split`

Each branch should:

- avoid behavior changes
- land in small mergeable slices
- stop after one hotspot area instead of chaining multiple refactors together

## What To Avoid

- a single mega-branch that splits all four big Rust files
- feature work in the middle of a hotspot refactor branch
- splitting files by arbitrary line count instead of by responsibility
- over-fragmenting authored YAML into dozens of tiny files before ownership patterns are clear

## Bottom Line
You do not need to slow down to fully sequential work.

You do need a stricter rule for when branches are truly independent, and you should gradually split the current hotspot files so "independent" stops meaning "they both still edited `src/app.rs` anyway."
