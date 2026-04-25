# Contains Rules and Intended Behaviours

Reference document for the authored hierarchy schema.
Use this while designing inference rules and authoring new data.

---

## Overview

The hierarchy has six levels: template, group, section/collection, field, list, item.
Each level has a `contains` (or equivalent) that determines what it accepts.
The runtime infers behaviour from what's present - no explicit `body:` or `type:` needed.

All levels are validated at load time. Invalid child kinds are a hard error, and `item.fields`
now resolves against authored field IDs rather than acting as an unchecked string list.

**Global rule:** no level may contain a `template`. There is exactly one template and it is the root.

**(later):** all "explicit no" conditions should produce a clear, specific error message telling
the author exactly what is not allowed and why, rather than a generic parse failure.

---

## `template`

**Parents:** none - template is the root

**Accepts:** `group` refs only; `section` refs (next); `collection` refs (later)

**Inferred body:** none - template is a structural root only, it has no render mode of its own.

```yaml
template:
  contains:
    - group: some_group_id
```

**(next/later):** `template.contains` should also accept `section` and `collection` refs directly.
Each bare section/collection gets a synthetic invisible group at runtime (no group heading in the map).
Ordering in `template.contains` is preserved exactly as written.
The section's own `nav_label` is unchanged - the invisible group just means no group-level heading above it.

---

## `group`

**Parents:** `template`; `group` (icebox)

**Accepts:** `section` refs, `collection` refs; `field` refs (later); nested `group` refs (icebox)

**Inferred body:** none - group is a nav cluster only, it has no render mode of its own.

**Cannot contain:** `field`, nested `group`, `template`
- `field`: (later) - runtime synthesizes an invisible section wrapper, useful when a single field
  doesn't warrant its own named section
- nested `group`: (icebox) - collapsible sub-groups in the nav map
- `template`: explicit no

```yaml
groups:
  - id: treatment_group
    contains:
      - section: treatment_section
      - collection: tx_regions
```

---

## `section`

**Parents:** `group`; `template` (next); `section` (icebox)

**Accepts:** `field` refs, `list` refs; `collection` refs (later)

**Cannot contain:** `collection`, `section`, `group`, `template`
- `collection`: (later) - useful but not urgent
- `section`: (icebox) - hard to imagine a use-case that isn't fiddly for the user
- `group`: explicit no
- `template`: explicit no

```yaml
sections:
  - id: appointment_section
    contains:
      - field: appointment_date_field
      - field: appointment_requested_field
```

### Inferred body from children

| Children | Inferred body | Behaviour |
|---|---|---|
| empty | `free_text` | open text entry, no modal - inferred correctly but no input UI exists yet (later) |
| fields only | `multi_field` | wizard with one slot per field |
| lists only | `list_select` | list/search picker |
| fields + lists | `multi_field` | lists get wrapped as pseudo-field slots alongside fields |
| collections only | TBD | (later) - probably mirrors `list_select` |
| fields + collections | `multi_field` | (later) - collections probably wrapped as pseudo-field slots, same as lists |
| lists + collections | TBD | (later) - needs definition |
| fields + lists + collections | `multi_field` | (later) - all wrapped as pseudo-field slots |

**Gaps / open questions:**
- `checklist` appears in the runtime dispatch (`maybe_record_section_lists`) but no authored
  path produces it. It is currently dead code. Intended behaviour TBD.
- `list_select` only has one runtime mode (`Browsing`). The `modal_start: search` property
  on a list is parsed and stored but not wired to the section's runtime mode.
- (later): `free_text` sections (empty `contains`) have no input mechanism - you can navigate
  to them but there is no way to actually enter text. Needs a text entry UI to be useful.
- unknown keys on authored blocks now hard-error during parse (for example `body: checklist`
  on a section). Common authoring mistakes now route through bespoke diagnostics, including
  unsupported authored keys, missing required keys, template-root mistakes, format placeholder
  mismatches, field cycles, and `assigns:` teaching messages.
- (next/#47): in `fields + lists` sections, lists are stored in `SectionConfig.lists` but the
  `multi_field` renderer ignores them - they are silently dropped from the output. Intended
  behaviour needs to be defined and implemented.

---

## `collection`

**Parents:** `group`; `field`; `template` (next); `section` (later); `collection` (next)

**Accepts:** `list` refs only; `collection` refs (next)

**Cannot contain:** `field`, `section`, `group`, `template`
- `collection`: (next) - analogous to how groups treat sections; a collection containing
  collections groups them into a category. Behaviour at each nesting level will need to be
  thoroughly defined before implementation.
- `field`: explicit no (for now - revisit when nested collections are defined)
- `section`: explicit no
- `group`: explicit no
- `template`: explicit no

```yaml
collections:
  - id: tx_regions
    contains:
      - list: back
      - list: neck
```

Collections are synthesized into a `collection` body at runtime - this is the only body
type that is not inferred from children; it comes from being declared as a `collection`
block rather than a `section` block.

### Inferred body from children

| Children | Inferred body | Behaviour |
|---|---|---|
| empty | none | valid YAML but renders empty - needs definition |
| lists only | `collection` | current behaviour - list-backed collection picker |
| collections only | TBD | (next) - nested collection grouping, behaviour needs thorough definition |
| lists + collections | TBD | (next) - mixed case, needs definition |

**Gaps / open questions:**
- A collection with an empty `contains` is valid YAML but would render as empty.

---

## `field`

**Parents:** `section`; `field` sub-field; `item` (via `fields:`); `group` (later)

**Accepts in `contains`:** `field` refs, `list` refs, `collection` refs

**Legacy parallel syntax:** `lists: [id, id]`, `collections: [id, id]` - rejected with a
clear parse error pointing the author to use `contains:` instead.

**Cannot contain:** `section`, `group`, `template`
- `section`: explicit no
- `group`: explicit no
- `template`: explicit no

```yaml
fields:
  - id: requested_regions
    label: Requested Regions
    format: "{single_region}"
    joiner_style: comma_and_the
    max_entries: 6
    contains:
      - field: single_region
```

### Inferred behaviour from children

| Children | Attributes | Runtime behaviour |
|---|---|---|
| empty, no format | - | bare label / text slot (no picker). Currently unvalidated - behaviour is renderer-dependent. |
| empty, with format | - | format-driven: `{placeholder}` IDs are resolved as `format_lists` automatically |
| lists only | - | one picker slot per list |
| lists only | `joiner_style` or `max_entries` | repeating picker - user adds multiple selections, output joined. (later): no live example, likely not working. |
| sub-fields only | - | composite field, each sub-field becomes its own modal slot |
| sub-fields + lists | - | composite, lists wrapped as pseudo-field slots alongside sub-fields |
| collection | - | collection-backed picker |
| sub-fields + collection | - | composite, collection wrapped as one slot. (later): untested, no live example. |
| sub-fields + `joiner_style` | - | repeating composite - each entry cycles through all sub-field slots, joined on output |

**Gaps / open questions:**
- A field with no children, no lists, and no format is not validated. Silently becomes
  a `HeaderFieldConfig` with empty everything. Intended behaviour is undefined.

---

## `list`

**Parents:** `section`, `collection`, `field` (all current)

**Contains:** explicitly nothing - if `contains:` is used on a list it should throw a useful error.

**Accepts:** `items` only, via the separate `items:` key. Items are inline definitions, not
references to other named blocks, so they use their own key rather than the typed child-ref system.
This distinction is intentional.

```yaml
lists:
  - id: the_region
    label: "region"
    modal_start: search
    joiner_style: comma_and_the
    max_entries: 3
    items:
      - id: shoulder
        label: "Shoulder"
        output: "Shoulder"
```

**Properties that affect runtime behaviour:**
- `modal_start: list | search` - browse mode vs search-first mode. Parsed and stored, but
  not yet wired end-to-end (see below).
- `joiner_style` - how repeated selections are joined in output
- `sticky` - remember chosen value across notes
- `default` - item selected by default
- `max_entries` - max number of repeat selections (same property as on fields)

**Gaps / open questions:**
- `repeat_limit` is no longer part of the active authored schema. Live YAML now uses
  `max_entries`; strict unknown-key validation rejects `repeat_limit` like any other
  unrecognised key.
- `modal_start: search` is authored and parsed but `ListSelectMode` only has one variant
  (`Browsing`), making it a no-op placeholder. (later): remove `ListSelectMode` or properly
  implement and wire search mode end-to-end.
- Items are not part of the typed child-ref / validation system - they're a parallel
  schema with different rules (see below).

---

## `item`

**Parents:** `list` only

Items are the leaf nodes. They don't use `contains`. They have their own parallel mechanism
for branching and side-effects.

```yaml
items:
  - id: appointment_type_treatment
    label: "Treatment massage, focusing on ..."
    output: "Treatment massage, focusing on "
    assigns:
      - list: some_other_list
        item: some_item_id
    fields: [field_id_one, field_id_two]       # item-driven branching
```

**Contains (functional, not via `contains:` key):**

`assigns:`
- Target: `list` + `item` only. Explicit no for all other targets (field values, higher levels, etc).
- When this item is chosen, pre-selects an item in another list.
- Validated: list and item IDs are checked. Self-assignment caught. Assign chains not cycle-checked.
- (next): figure out assigns scoping in detail. Current problem: a `pluralizer`
  list (handles "1 week" vs "2 weeks" etc.) breaks down when multiple pluralizers appear in the
  same entry (e.g. "every 1 day(s), starting 2 week(s) ago") because assigns are global and
  conflict depending on input order. Sub-field containment might scope each pluralizer instance
  independently, but assigns shouldn't always be scoped - needs proper design before implementation.
  This gets even more complicated on repeat behaviours, where the same field runs multiple times
  and each repetition may need its own isolated assign context.

`fields:`
- Intended to reference existing authored field blocks by ID and activate them as sub-fields
  when this item is chosen.
- Resolved and validated at load time. Missing IDs and wrong-kind IDs are hard errors.
- At runtime, the chosen authored fields are wired into the branch flow in authored order.
- `branch_fields` is no longer part of the authored schema; `fields:` is the supported path.

---

## Summary: where the schema is closed vs open

| Level | Closed? | Notes |
|---|---|---|
| template | Yes | fully validated |
| group | Yes | validated |
| section | Mostly | `checklist` dead; `list_select` search mode not wired |
| collection | Yes | list-only by design |
| field | Mostly | bare-field case unvalidated |
| list | Partially | `modal_start: search` parsed but not wired |
| item | Mostly | `fields` is validated and wired; assigns scoping still has open design work |
