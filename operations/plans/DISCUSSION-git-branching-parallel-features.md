# Discussion: Git Branching for Parallel Feature Work

**Date:** 2026-04-12
**Status:** Discussion Complete

## Plain-English Summary
Git branches let you work on multiple features at the same time without mixing them together.

For Scribblenot, the usual shape should be:

- `main` stays stable
- `feature/<name>` holds one focused change
- each feature branch merges back when it is ready

You do **not** need a heavily modularized codebase for branching to work. Branches work either way.

What modularization changes is merge friction:

- clear code boundaries make parallel work easier
- mixed responsibilities in large files make conflicts more likely

## Recommended Default
Use one branch per feature, keep branches short-lived, and merge them in small slices.

That is the safest default for this project right now.

Reason:

- the app already has some useful separation (`src/data.rs`, `src/note.rs`, `src/sections/*`)
- two files are still major coordination hotspots: `src/app.rs` and `src/ui.rs`
- large simultaneous edits in those hotspots will collide more often than edits in narrower modules

## Two Options

### Option 1: Branch First, Refactor Only When Pain Appears
Use focused feature branches on the current structure.

**Pros**
- fastest to start
- avoids speculative architecture work
- good fit for small and medium features

**Cons**
- branches that both touch `src/app.rs` or `src/ui.rs` may conflict often
- longer-running branches become harder to merge cleanly

### Option 2: Do Targeted Modularization Before or During Parallel Work
Split the most conflict-prone areas into smaller units first.

**Pros**
- fewer merge conflicts
- clearer ownership during feature work
- easier testing of narrow behavior

**Cons**
- costs time up front
- can become a rewrite if not tightly scoped
- risky if done broadly without a specific pain point

## Recommendation
Choose **Option 1 by default**, with **targeted modularization** only where branches repeatedly collide.

In plain terms: do not refactor the whole project just to "make Git work." Instead, let real friction tell you where boundaries are weak, then improve those spots deliberately.

## Practical Branch Strategy for Scribblenot

### Branch Types
- `main`: always releasable or close to it
- `feature/<short-name>`: normal feature work
- `fix/<short-name>`: bug fixes
- `refactor/<short-name>`: internal cleanup with no intended behavior change

Examples:

- `feature/modal-stream-polish`
- `feature/clipboard-import`
- `fix/note-heading-regression`
- `refactor/app-event-routing`

### Branch Rules
- one branch should answer one question
- avoid mixing product changes and cleanup in the same branch
- if cleanup is required first, give it its own `refactor/` branch and merge it early
- prefer several small mergeable branches over one long-running branch

## Example: Three Simultaneous Features
Suppose you want all three at once:

1. improve modal rendering
2. add a YAML validation command
3. adjust note export formatting

Recommended branch split:

- `feature/modal-rendering`
- `feature/yaml-validation`
- `feature/export-formatting`

This works well because the likely ownership is mostly separate:

- modal rendering mainly hits `src/ui.rs`, `src/modal.rs`, and maybe `src/app.rs`
- YAML validation mainly hits `src/data.rs`, maybe `src/main.rs`, and tests
- export formatting mainly hits `src/note.rs` and tests

If two branches both need `src/app.rs`, that is still manageable. The goal is not "zero overlap." The goal is "small, understandable overlap."

## Example: Overlapping Modal UI Work
This is the case that usually feels confusing at first, because the branches are close together in both code and product behavior.

Suppose you want to do all three:

1. improve modal transitions
2. change how collection preview modals behave
3. add another modal UX tweak, such as different hint treatment or card emphasis

These are all adjacent. They will likely touch some combination of:

- `src/modal.rs`
- `src/ui.rs`
- `src/app.rs`

That does **not** mean they must all live in one branch.

The key question is:

Can one change make sense on its own, with the next change building on top of it?

If yes, use **stacked branches**.

### Recommended Split

Base branch:

- `feature/modal-transitions-foundation`

Stacked on top of it:

- `feature/collection-preview-polish`

Stacked on top of that, or sometimes directly on the foundation branch:

- `feature/modal-ux-tweak`

### What Each Branch Should Hold

`feature/modal-transitions-foundation`
- transition state
- timing/easing behavior
- render/update plumbing required for animated movement
- no unrelated preview policy changes unless they are strictly required

`feature/collection-preview-polish`
- collection preview card behavior
- preview-specific layout/content decisions
- rules for what neighboring or teaser cards should show

`feature/modal-ux-tweak`
- isolated presentational or interaction polish
- for example, hint visibility, emphasis, spacing, or focus treatment

### Why This Split Helps
- the transition branch is the foundation
- the preview branch can assume that foundation exists
- later polish branches stay smaller and easier to reason about
- if the foundation is good, it can merge first and reduce conflict for the later branches

### Suggested Merge Order

1. merge `feature/modal-transitions-foundation`
2. rebase or merge `main` into `feature/collection-preview-polish`
3. merge `feature/collection-preview-polish`
4. rebase or merge `main` into `feature/modal-ux-tweak`
5. merge `feature/modal-ux-tweak`

### When To Keep It As One Branch Instead
Keep adjacent modal work in one branch if the earlier branch would be incomplete or misleading on its own.

Examples:

- the transition machinery only exists to support one very specific preview behavior
- the preview change would look broken without the exact follow-up UX tweak
- separating the work would create temporary states you would never actually want merged

In those cases, one branch is cleaner than artificial separation.

### Rule Of Thumb For UI Branches
If the work is:

- **foundation first, behavior second** -> split into stacked branches
- **one inseparable user-facing change** -> keep one branch

For Scribblenot modal work, the best default is usually:

- one small branch for modal infrastructure
- one follow-up branch for collection-preview behavior
- one optional polish branch after that

That gives you sequencing without pretending the work is fully independent.

## Merge Order
Merge the branch with the cleanest boundary and lowest risk first.

A useful order is:

1. internal refactors that unblock other work
2. smallest isolated feature
3. higher-risk UI or workflow changes

Why this helps:

- later branches can rebase onto a cleaner base
- shared scaffolding lands once instead of being duplicated

## Keeping Multiple Branches Healthy
If a feature branch stays open more than a day or two, update it from `main` regularly.

Two valid approaches:

### Rebase onto `main`
Best when you want a clean, linear branch history.

```powershell
git switch feature/modal-rendering
git fetch
git rebase main
```

### Merge `main` into the branch
Best when you want the safest, least history-rewriting workflow.

```powershell
git switch feature/modal-rendering
git fetch
git merge main
```

For a solo workflow, either is fine. If you are still getting comfortable with Git, merging `main` into a branch is usually simpler and less stressful.

## When Conflicts Happen
Conflicts do **not** mean branching failed. They mean two branches changed the same area.

Use this decision rule:

- if both branches are changing the same behavior, finish and merge one first
- if both branches are touching the same file for unrelated reasons, look for a local extraction that separates responsibilities
- if conflicts keep repeating in the same file, that file is a refactor signal

## What Modularization Would Help Most Here
Scribblenot does not need a broad rewrite for parallel branch work, but it does have obvious hotspots.

Current likely hotspots:

- `src/ui.rs` at about 4,153 lines
- `src/app.rs` at about 3,497 lines
- `src/data.rs` at about 1,742 lines

That suggests a practical modularization path:

### Good Target 1: UI rendering slices
Split `src/ui.rs` by concern, for example:

- layout and pane composition
- modal rendering
- status/help overlays
- per-section widget rendering

### Good Target 2: App event handling
Split `src/app.rs` by behavior, for example:

- global key dispatch
- modal actions
- section navigation
- persistence/reload actions

### Good Target 3: Data loading and validation
Separate:

- YAML deserialization shapes
- reference resolution
- validation/reporting

These boundaries help both maintainability and branch isolation.

## What Not To Do
- do not make a long-lived "mega branch" for several features at once
- do not hide behavior changes inside a "refactor" branch
- do not do a large architecture rewrite unless branch pain clearly justifies it
- do not leave branches stale for a long time if `main` is moving

## Recommended Working Pattern
For this project, a good default loop is:

1. branch from current `main`
2. make one coherent change
3. keep commits small and readable
4. merge as soon as the branch is valid
5. update remaining branches from new `main`
6. only refactor hotspots that are causing repeated conflict

## Bottom Line
Branches solve **parallel work tracking**.

Modularization solves **parallel code ownership**.

You need branches immediately.
You only need modularization when the same areas of code keep colliding or becoming hard to reason about.

For Scribblenot, the best next step is not a general rewrite. It is a disciplined branch workflow plus selective extractions from `src/ui.rs` and `src/app.rs` when those files start slowing feature work down.
