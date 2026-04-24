# Bespoke Diagnostic Teaching Moments

## Why this note exists

`missing_child` and `wrong_kind_reference` already earn their extra space because they usually point at a real authoring decision, not just a bad parser state.

`invalid_child_kind` is even more valuable when it explains a project-specific rule and offers the intended interaction shape.

This note proposes the next small set of diagnostics that seem worth that same treatment.

## Selection rule

A category should get bespoke copy only when the error is doing at least one of these jobs:

- teaching a Scribblenot-specific schema rule
- explaining an inferred behaviour that is not obvious from the YAML alone
- showing a small workflow, not just a corrected token

That means plain lookup failures like `duplicate_id`, `invalid_hotkey`, or ordinary `missing_child` cases usually do not need more than the current structured copy.

## Best candidates

### 1. The `looks_like_*` family

Categories:

- `looks_like_list_missing_items`
- `looks_like_collection_missing_key`
- `looks_like_section_or_group_missing_key`

Why these are teaching moments:

- They are not really "wrong kind" errors.
- They usually mean "this block already looks like the thing you meant, but it lost the key that makes that kind valid".
- The fingerprint logic is clever, but the author does not know that unless the message explains it.

What the bespoke copy should teach:

- Scribblenot inferred the likely intent from kind-specific fields.
- A list needs `items:`.
- A section, group, or collection needs `contains:`.
- The top-level bucket also matters: `lists:`, `collections:`, `sections:`, `groups:`.

Suggested interaction style:

- Start with "This block looks like a list/collection/section/group."
- Name the fingerprint fields that made it look that way.
- Show two fixes:
  - restore the missing structural key
  - move the block under the correct top-level key

Why it feels worth it:

- This is exactly the kind of error where a user can feel "I almost had it right".
- A bespoke explanation turns a confusing type mismatch into a concrete repair.

Example:

```yaml
sections:
  - id: intake_section
    contains:
      - list: appointment_type

fields:
  - id: appointment_type
    label: Appointment Type
    modal_start: search
    sticky: true
```

Example bespoke explanation:

> `appointment_type` is being used as a list, but this block looks like a list that is missing its `items:` key.
>
> Scribblenot guessed that from list-only fields on the block: `modal_start` and `sticky`.
>
> To fix it, either:
> 1. keep this as a list by adding `items:` and moving it under top-level `lists:`
> 2. or change the reference in `intake_section` so it points at a real field instead

This is stronger than a plain wrong-kind message because it explains why Scribblenot inferred the likely intent.

### 2. The template-structure family

Categories:

- `missing_template`
- `multiple_templates_across_files`
- `template_count_invalid`
- `template_runtime_child_invalid`

Why these are teaching moments:

- `template` is a Scribblenot root concept, not generic YAML knowledge.
- Authors need to learn that there must be exactly one template across the whole data set.
- The allowed child rule is also product-specific: today `template.contains` is a `group` list, not a general hierarchy entry point.

What the bespoke copy should teach:

- The template is the single root that defines top-level navigation order.
- Exactly one `template:` block must exist across all hierarchy files.
- Today, template children should be `group` references.

Suggested interaction style:

- Show a minimal valid template snippet.
- If there are too many templates, show "pick one root and merge the other `contains:` entries into it".
- If the child kind is invalid, explain the current limitation plainly rather than only saying "must be group references".

Why it feels worth it:

- This is usually the author's first schema-level mistake.
- A good explanation would save a lot of rereading across files.

Example:

```yaml
template:
  contains:
    - section: subjective_section

sections:
  - id: subjective_section
    contains: []
```

Example bespoke explanation:

> The `template` block is the single root of the hierarchy.
>
> Right now, `template.contains` should list `group` references, not sections directly.
>
> A minimal valid shape looks like:
>
> ```yaml
> template:
>   contains:
>     - group: intake
>
> groups:
>   - id: intake
>     contains:
>       - section: subjective_section
> ```
>
> Put `subjective_section` inside a group, then reference that group from the template.

This teaches the current root structure instead of only reporting that the child kind is invalid.

### 3. The format-placeholder family

Categories:

- `field_unknown_format_list`
- `field_expected_format_list_wrong_kind`
- `field_unknown_explicit_format_list`
- `field_explicit_format_list_wrong_kind`
- `runtime_unknown_format_list`

Why these are teaching moments:

- `format`, `{placeholders}`, and `format_lists` are a special Scribblenot authoring model.
- When these fail, the real problem is often not "unknown list"; it is "I do not understand where placeholder values come from".

What the bespoke copy should teach:

- A placeholder like `{year}` resolves through a list attached to the field.
- That attached list can come from `format_lists:` or the field's broader child setup, depending on the path.
- The placeholder name and the attached list id must line up.

Suggested interaction style:

- Show a tiny before/after example with one placeholder and one list id.
- Explain the resolution path in one sentence:
  - "`format` asks for `{year}` -> the field must expose a list with id `year`."

Why it feels worth it:

- These errors are concept-heavy and easy to misread as arbitrary loader strictness.
- A short teaching example would likely prevent repeat mistakes.

Example:

```yaml
fields:
  - id: duration_field
    label: Duration
    format: "{year}"
    format_lists:
      - years
```

Imagine the author really meant `{years}` or the list id really should have been `year`.

Example bespoke explanation:

> `duration_field` uses the placeholder `{year}`, but this field does not expose a list with id `year`.
>
> In Scribblenot, placeholders in `format` resolve through lists attached to the same field.
>
> That means this field needs a matching pair:
>
> ```yaml
> format: "{year}"
> format_lists:
>   - year
> ```
>
> If the list id is `years`, then update the placeholder to `{years}` instead.

This teaches the placeholder-to-list contract, which is the part authors are least likely to infer on their own.

### 4. `runtime_field_cycle`

Why this is a teaching moment:

- A cycle is not just a bad reference; it is a bad interaction graph.
- The author needs to understand why the wizard cannot ever finish that field tree.

What the bespoke copy should teach:

- Show the cycle path if possible, for example `a -> b -> c -> a`.
- Explain that each field eventually asks for itself again.
- Offer two concrete escape routes:
  - remove one `contains:` edge
  - flatten one step into a list or format-driven value instead of a nested field

Why it feels worth it:

- Without the path, "field cycle" is correct but not very actionable.
- This is a classic graph problem that benefits from visual wording.

Example:

```yaml
fields:
  - id: pain_summary
    label: Pain Summary
    contains:
      - field: pain_region

  - id: pain_region
    label: Pain Region
    contains:
      - field: pain_detail

  - id: pain_detail
    label: Pain Detail
    contains:
      - field: pain_summary
```

Example bespoke explanation:

> These field references form a cycle:
>
> `pain_summary -> pain_region -> pain_detail -> pain_summary`
>
> That means the wizard can never finish walking this field tree, because completing `pain_detail` would ask for `pain_summary` again.
>
> Remove one of these `contains:` links, or flatten one step into a list/format-driven value instead of another nested field.

This turns an abstract graph error into something the author can actually trace and repair.

### 5. The `assigns` family

Categories:

- `assign_self_reference`
- `assign_unknown_list`
- `assign_unknown_item`
- `runtime_assign_unknown_list`
- `runtime_assign_unknown_item`

Why these are teaching moments:

- `assigns:` is a bespoke interaction mechanic.
- These errors are really about a side-effect workflow: choosing one item should auto-apply choices somewhere else.

What the bespoke copy should teach:

- The source item lives in one list.
- Its `assigns:` block points to a target list and target item.
- Scribblenot uses that target to pre-fill or apply downstream values.

Suggested interaction style:

- Describe the flow as "when this item is chosen, also select that item in that list".
- Name both ends of the relationship.
- If the list or item is missing, show which side is broken.

Why it feels worth it:

- These errors are easiest to fix once the author sees the relationship as an action, not just a broken reference.

Example:

```yaml
lists:
  - id: side_list
    label: Side
    items:
      - id: left
        label: Left
        output: left
        assigns:
          - list: laterality_words
            item: lt

  - id: laterality_words
    label: Laterality Words
    items:
      - id: left_word
        label: Left
        output: left
```

Example bespoke explanation:

> When `left` is chosen in `side_list`, this item tries to also select `lt` in `laterality_words`.
>
> That target item does not exist. The available item here is `left_word`.
>
> In Scribblenot, `assigns:` is a side-effect rule:
> "when this item is chosen, also choose that item in another list."
>
> Fix the target item id, or remove the assignment if this side effect is no longer needed.

This teaches `assigns:` as a workflow relation, which is much easier to reason about than a bare missing-id report.

## Lower-priority candidates

### `legacy_repeating_key`

This is probably worth bespoke copy only if migration errors become common.

Why it is borderline:

- It is a migration/rename moment, so a before/after example would help.
- But it is narrower than the families above and not as conceptual once the rename is known.

If upgraded later, the message should teach:

- `repeating:` is old vocabulary
- `joiner_style:` is the current authored knob
- a small before/after snippet is probably enough

## What I would not bespoke yet

- `duplicate_id`
- `duplicate_boilerplate_id`
- `missing_child`
- `wrong_kind_reference`
- `invalid_hotkey`
- file I/O failures

Reason:

- these are already understandable with the current structured wording
- extra bespoke copy would add weight without teaching much new

## Recommended order

1. `looks_like_*`
2. template-structure family
3. format-placeholder family
4. `runtime_field_cycle`
5. `assigns` family

That ordering keeps the effort focused on the places where the message is explaining Scribblenot's mental model, not just its validation rules.
