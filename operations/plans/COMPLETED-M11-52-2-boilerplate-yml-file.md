## Task

#52 - Extract hard-coded boilerplate strings from note.rs into editable YML data files

## Context

Sub-task 52.1 added the `FlatBlock::Boilerplate` variant to `src/flat_file.rs`. Sub-task 52.2 creates the actual data file that holds the two hard-coded boilerplate strings currently embedded in `src/note.rs`:

1. The treatment note boilerplate (line 164) -- emitted as the opening lines of `## TREATMENT / PLAN`.
2. The informed consent statement (line 156) -- emitted as the body of `#### INFORMED CONSENT` in `## SUBJECTIVE`.

The boilerplate text values are extracted from their surrounding rendering context in note.rs (headers, bullet prefixes, leading/trailing newlines added by the note assembler are not stored in the YML). Specifically: `informed_consent` omits the `- ` list bullet (rendering concern for the loader), and `treatment_plan_disclaimer` gains a single trailing `\n` from the YAML `|` block scalar that is not in the current source string. The loader sub-task must account for these differences so runtime note output is unchanged.

## Approach

Create `data/boilerplate.yml` containing exactly two `FlatBlock::Boilerplate` blocks using the `type: boilerplate` discriminant established in sub-task 52.1. Use YAML block scalars (`|`) to preserve the internal newlines exactly. No Rust changes are needed in this sub-task -- the file is created now so a later sub-task can wire the loader.

## Critical Files

- `src/note.rs` line 156: informed consent string literal (source of truth for text)
- `src/note.rs` line 164: treatment note boilerplate string literal (source of truth for text)
- `src/flat_file.rs` lines 38-41: `FlatBlock::Boilerplate { id: String, text: String }` definition
- `data/` directory: location for the new file (alongside `sections.yml`, `tx_mods.yml`, etc.)

## Reuse

- `FlatBlock::Boilerplate` variant already defined in `src/flat_file.rs` -- no new Rust code needed.
- YAML format follows the same `blocks:` list structure used by all other files in `data/`.

## Steps

1. Create `data/boilerplate.yml` with the following exact content:

```yaml
blocks:
  - type: boilerplate
    id: treatment_plan_disclaimer
    text: |
      Regions and locations are bilateral unless indicated otherwise.
      Patient is pillowed under ankles when prone, and under knees when supine.

  - type: boilerplate
    id: informed_consent
    text: Patient has been informed of the risks and benefits of massage therapy, and has given informed consent to assessment and treatment.
```

Note: `treatment_plan_disclaimer` uses a YAML block scalar (`|`) because the string contains an embedded newline. `informed_consent` is a single line and needs no block scalar.

## Verification

### Manual tests

- Open `data/boilerplate.yml` and verify:
  - There are exactly two blocks, both with `type: boilerplate`.
  - `treatment_plan_disclaimer` text matches the body of the note.rs line 164 string (excluding the `## TREATMENT / PLAN\n` header prefix), plus one trailing `\n` added by the YAML `|` block scalar.
  - `informed_consent` text matches the body of the note.rs line 156 string (excluding the `- ` list bullet prefix used during note assembly).
- Run `cargo build` and confirm it compiles without errors or new warnings (the file is not yet wired to the loader, so no behavioral change is expected).

### Automated tests

- A unit test in `src/flat_file.rs` (or a dedicated integration test) that calls `serde_yaml::from_str` on the content of `data/boilerplate.yml` and asserts:
  - The resulting `FlatFile` contains exactly two blocks.
  - Block 0 is `FlatBlock::Boilerplate { id: "treatment_plan_disclaimer", text: "Regions and locations are bilateral unless indicated otherwise.\nPatient is pillowed under ankles when prone, and under knees when supine.\n" }`.
  - Block 1 is `FlatBlock::Boilerplate { id: "informed_consent", text: "Patient has been informed of the risks and benefits of massage therapy, and has given informed consent to assessment and treatment." }`.

## Implementation
Complete - 2026-04-02

## Progress
- Step 1: Created data/boilerplate.yml with two boilerplate blocks (treatment_plan_disclaimer and informed_consent)

## Changelog

### Review - 2026-04-02
- #1: Replaced inaccurate "byte-for-byte identical" claim in Context with precise description of how each string differs from the source literal (trailing `\n` added by YAML block scalar for treatment_plan_disclaimer; `- ` bullet omitted from informed_consent), and updated Manual tests accordingly so the loader sub-task is not misled.
