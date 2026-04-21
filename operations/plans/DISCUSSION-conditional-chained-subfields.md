# Discussion: Conditional Chained Sub-fields

**Date:** 2026-04-20
**Status:** Discussion Complete

## Problem
Some form fields have options that require additional input to be meaningful (e.g. selecting "every" requires a number and a unit to form "every 3 weeks"). Today, either those sub-fields are always visible (clutter) or the user can submit an incomplete value (bad state). With 5-10+ such fields across the app, a scalable pattern is needed.

## Goal
When a trigger option is selected in a field, smoothly collect required sub-field values through a focused sub-modal, then return a fully composed value to the parent field. Reduce clutter for the common case while preventing incomplete entries.

## Core Concept
A "call stack" modal pattern: the parent field strip slides up out of view, a sub-field strip slides into its place. The user fills out the sub-field (which may itself trigger further sub-fields, chaining as needed). As each level is confirmed, the strip slides back down and the parent's composition box receives a single merged phrase representing the resolved chain.

## Context & Users
Clinicians filling out structured medical notes in a frequently-repeated workflow. Because the workflow is repeated often, users will learn the interaction quickly - but the slide animation provides clear spatial feedback at every level of the chain so the current "depth" is always legible.

## Constraints & Anti-goals
- Chains must be supported - a sub-field can itself trigger further sub-fields (no artificial depth limit)
- The preview-pane approach (always-visible inactive sub-fields) is explicitly ruled out - it does not scale with chains
- No separate confirm button on sub-fields - the live composition box acts as the running review; confirmation is implicit when the field is complete
- While inside a sub-field chain, values display as individual tokens (editable); once resolved back to the parent, they collapse into a single merged phrase (e.g. `every 3 weeks`)

## Key Decisions
- **Slide vs. preview pane:** Preview pane was considered but rejected because inactive chain levels would stack up and create noise in a repeated workflow.
- **Sub-field confirmation UX:** No explicit confirm step inside sub-fields - the auto-filling composition box provides continuous feedback, and returning to the parent level signals acceptance.
- **Token vs. merged display:** Tokens remain discrete while composing inside a sub-field chain; they merge into a phrase once the chain resolves back to the parent. This keeps editing granular where it matters and output clean where it doesn't.

## Open Questions
- How are conditional trigger relationships defined in the YAML config? (e.g. a `triggers` key on a list option pointing to a sub-field/list id)
- Should there be a way to "undo" a resolved sub-field from the parent - i.e. re-enter the chain to edit individual tokens after they've merged?
- How does the slide animation behave when a chain is 3+ levels deep - does each level stack visually or fully replace the previous strip?
- Should the back/cancel gesture at any point in the chain reset only the current level or the entire chain?
