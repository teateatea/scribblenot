# Implementation Brief: `src/data.rs` Split Scaffold

**Date:** 2026-04-24
**Status:** Proposed
**Scope:** Slice 0 only

## Purpose
Define the smallest safe first implementation step for the `src/data.rs` hotspot split.

This brief is intentionally narrow. It does **not** authorize moving loader, validation, or runtime logic yet. Its job is only to create the scaffolding that later slices can use.

## Plain-English Summary
Before moving real logic out of `src/data.rs`, the codebase needs clean landing zones.

The safest first move is:

- keep `src/data.rs` as the public entrypoint
- add explicitly named helper files next to it
- wire those helpers in without changing behavior

That gives later branches somewhere clear to put moved code, while keeping this first branch small and easy to merge.

## Recommendation
Use one very small refactor branch for this step:

- branch name: `refactor/data-split-scaffold`

The branch should compile, change no behavior, and mostly prepare structure.

## In Scope

- add new helper files:
  - `src/data_model.rs`
  - `src/data_hints.rs`
  - `src/data_source.rs`
  - `src/data_load.rs`
  - `src/data_runtime.rs`
  - `src/data_validate.rs`
- declare those modules from `src/main.rs`
- make `src/data.rs` the facade/orchestration file that can later re-export from them
- add minimal file headers or comments describing intended ownership

## Out of Scope

- moving large function bodies
- changing signatures
- renaming public types
- changing test behavior
- changing YAML loading order
- changing validation or diagnostics

## Exact structural target

### `src/main.rs`
Add new module declarations for the helper files so `src/data.rs` can use them.

Expected shape:

```rust
mod data;
mod data_hints;
mod data_load;
mod data_model;
mod data_runtime;
mod data_source;
mod data_validate;
```

This keeps module ownership explicit and avoids introducing a new `src/data/mod.rs` facade.

### `src/data.rs`
Remain the only public facade for callers using `crate::data::*`.

During Slice 0 it should:

- keep all current behavior
- begin importing from helper files only where that costs almost nothing
- avoid becoming half-facade, half-new-abstraction if the wiring gets awkward

The success condition is not "move lots of code." The success condition is "create a stable place for later code moves."

### New helper files
Each new file should start with a short header comment stating its intended future ownership.

Example style:

```rust
// Data model helpers extracted from data.rs.
// Slice 0 scaffold only: logic will move here in later refactor slices.
```

The files may be mostly empty in Slice 0. That is acceptable.

## Two Options

### Option 1: Create empty helper files only
Add the files and module declarations, but do not move or re-export anything yet.

**Pros**
- smallest possible change
- almost zero behavior risk
- easiest merge

**Cons**
- `src/data.rs` still looks fully monolithic right after the branch lands
- later slices still need to do the first visible wiring step

### Option 2: Create helper files and move one trivially isolated item
Add the files and also move one very low-risk isolated item, such as a comment-only ownership header or a tiny hint helper.

**Pros**
- proves the wiring pattern
- gives the next slice a concrete example

**Cons**
- slightly more review surface
- increases the chance of accidental scope creep

## Recommendation
Choose **Option 1** unless a truly trivial move falls out naturally during implementation.

This branch should optimize for safety and mergeability, not visible progress. The real progress starts in the next slice.

## Suggested ownership sentence
`This branch owns internal module scaffolding for the future data split. It should not change loader behavior, validation behavior, runtime construction, or public APIs.`

## Review checklist

- all new filenames are explicit and prefixed with `data_`
- `src/data.rs` remains the public entrypoint for callers
- no new `mod.rs` file is introduced
- no public behavior changes
- no opportunistic cleanup outside the scaffold scope

## Validation

Run:

```powershell
cargo check --quiet
```

Optional:

```powershell
cargo test --quiet
```

`cargo test --quiet` is useful if the branch review reveals any unexpected signature churn, but it should not be required if Slice 0 remains purely structural.

## Done condition
Slice 0 is done when:

- the new helper files exist
- the crate compiles
- `src/data.rs` is still the public facade
- a later branch can begin moving logic into the helper files without redoing the scaffolding
