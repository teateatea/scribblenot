## Task
#52 - Extract hard-coded boilerplate strings from note.rs into editable YML data files

## Context
`render_note` in `src/note.rs` embeds two boilerplate strings as Rust literals:

- Line 158: the INFORMED CONSENT block (hard-coded inline in the `subj_parts.push(...)` call)
- Line 166: the TREATMENT / PLAN disclaimer ("Regions and locations are bilateral unless indicated otherwise. Patient is pillowed under ankles when prone, and under knees when supine.")

Both strings already exist in `data/boilerplate.yml` under the IDs `informed_consent` and `treatment_plan_disclaimer`. The `render_note` and `section_start_line` function signatures already accept `boilerplate_texts: &HashMap<String, String>`, so the plumbing is in place. The only remaining step is to replace the literals with runtime lookups. Three TDD tests at line 676-736 of `src/note.rs` enforce this: custom text injected via the map must appear; the old literals must not.

## Approach
Replace each hard-coded literal with a `boilerplate_texts.get("key").map(|s| s.as_str()).unwrap_or("")` lookup. When the key is missing the text is omitted (empty string), satisfying the `empty_boilerplate_map_does_not_silently_use_hard_coded_strings` test. No new abstractions are needed.

## Critical Files
- `src/note.rs` lines 155-166: the two hard-coded strings inside `render_note`

## Reuse
- `boilerplate_texts: &HashMap<String, String>` - already passed into `render_note`; no signature changes required.

## Steps

1. Replace the hard-coded INFORMED CONSENT push (line 158) with a boilerplate lookup.

```
-    subj_parts.push("\n\n\n#### INFORMED CONSENT\n- Patient has been informed of the risks and benefits of massage therapy, and has given informed consent to assessment and treatment.".to_string());
+    let informed_consent = boilerplate_texts.get("informed_consent").map(|s| s.as_str()).unwrap_or("");
+    if !informed_consent.is_empty() {
+        subj_parts.push(format!("\n\n\n#### INFORMED CONSENT\n- {}", informed_consent));
+    }
```

2. Replace the hard-coded TREATMENT / PLAN disclaimer (line 166) with a boilerplate lookup.

```
-    tx_parts.push("\n\n## TREATMENT / PLAN\nRegions and locations are bilateral unless indicated otherwise.\nPatient is pillowed under ankles when prone, and under knees when supine.".to_string());
+    let tx_disclaimer = boilerplate_texts.get("treatment_plan_disclaimer").map(|s| s.as_str()).unwrap_or("");
+    let tx_header = if tx_disclaimer.is_empty() {
+        "\n\n## TREATMENT / PLAN".to_string()
+    } else {
+        format!("\n\n## TREATMENT / PLAN\n{}", tx_disclaimer)
+    };
+    tx_parts.push(tx_header);
```

3. Run the tests to confirm all three new boilerplate tests pass and no existing tests regress.

```
cargo test -p scribblenot 2>&1
```

## Verification

### Manual tests
- None required; the change is fully covered by automated tests.

### Automated tests
- `cargo test -p scribblenot` must pass with zero failures.
- Specifically verify the three new tests introduced in sub-task 52.5:
  - `informed_consent_text_comes_from_boilerplate_map`
  - `treatment_plan_disclaimer_comes_from_boilerplate_map`
  - `empty_boilerplate_map_does_not_silently_use_hard_coded_strings`
- All pre-existing `note::tests::*` tests must also continue to pass.

## Progress
- Step 1: Replaced hard-coded INFORMED CONSENT push with boilerplate_texts.get("informed_consent") lookup
- Step 2: Replaced hard-coded TREATMENT / PLAN disclaimer with boilerplate_texts.get("treatment_plan_disclaimer") lookup
- Step 3: All 122 tests pass (including 3 boilerplate-specific tests)

## Implementation
Complete - 2026-04-03
