## Task
#45 - Refactor data format to flat, type-tagged YML blocks with ID-based cross-references

## Context
Sub-tasks 4 and 5 migrated all data files to flat type-tagged format and extended `FlatBlock` variants with metadata fields (`name`, `map_label`, `section_type`, `num` for groups/sections; `entries` for options-list). However, those sub-tasks left all metadata fields blank: every section block in `sections.yml` still has only `type:` and `id:`, and every options-list file still has an empty stub block with no `entries:`. The original data - group names, section names, map_labels, section_types, data_file links, and full option entries - lives in git history at commit `69ef14a` and must be restored into the flat format. Until this is done, `AppData.load` returns empty groups/sections and all list/checklist/region data is missing, so the app renders nothing useful.

`FlatBlock::Section` also needs a `data_file: Option<String>` field added before the data files can express which sections reference external option files. This extends ST5's work on the same variant.

## Approach
Two phases: first extend `FlatBlock::Section` in `src/flat_file.rs` with a `data_file: Option<String>` field (and a `date_prefix: Option<bool>` field - also needed by the original data). Then populate every data file block with its recovered metadata. The `sections.yml` section and group blocks get `name:`, `map_label:`, `section_type:`, and where applicable `data_file:` and `date_prefix:`. The options-list files (`tx_mods.yml`, `objective_findings.yml`, `remedial.yml`, `infection_control.yml`, `muscles.yml`) get their full `entries:` lists restored from git history. The `tx_regions.yml` file remains a stub `options-list` block for now - its `regions:` structure is not representable as `Vec<PartOption>` and requires a dedicated block type; that is a follow-on concern. No Rust loader logic changes are needed beyond the `FlatBlock::Section` extension.

## Critical Files
- `src/flat_file.rs` lines 16-22 - `Section` variant definition; add `data_file` and `date_prefix` fields
- `src/flat_file.rs` lines 67-70, 99-100 - test struct literals for `Section`; must include new fields
- `data/sections.yml` - add `name:`, `map_label:`, `section_type:`, `data_file:`, `date_prefix:` to all section/group blocks
- `data/tx_mods.yml` - restore full `entries:` list (36 entries from `69ef14a:data/tx_mods.yml`)
- `data/objective_findings.yml` - restore full `entries:` list (24 entries from `69ef14a:data/objective_findings.yml`)
- `data/remedial.yml` - restore full `entries:` list (14 entries from `69ef14a:data/remedial.yml`)
- `data/infection_control.yml` - restore as `type: options-list` with entries from original `items:` list (5 items, each as `{ label: "...", output: "..." }`)
- `data/muscles.yml` - restore full `entries:` list (51 entries from `f664b7f:data/muscles.yml`)
- `data/tx_regions.yml` - no change (stays as stub; regions format is not yet representable)

## Reuse
- `PartOption::Labeled { label, output }` shape from `src/data.rs` line 15 - the `{ label: "...", output: "..." }` YAML form used by all existing options files; infection_control items have identical label and output so use the `Labeled` shape
- `#[serde(default)]` pattern used on every other optional field in `FlatBlock` - apply same to new fields
- `Option<String>` / `Option<bool>` field patterns already established in `FlatBlock::Section` at lines 20-22

## Steps

1. Add `data_file: Option<String>` and `date_prefix: Option<bool>` to the `Section` variant in `src/flat_file.rs`.

```diff
     Section {
         id: String,
         #[serde(default)] children: Vec<String>,
         #[serde(default)] name: Option<String>,
         #[serde(default)] map_label: Option<String>,
         #[serde(default)] section_type: Option<String>,
+        #[serde(default)] data_file: Option<String>,
+        #[serde(default)] date_prefix: Option<bool>,
     },
```

2. Update the two `FlatBlock::Section` struct literals in `src/flat_file.rs` tests to include the two new fields with `None` defaults (lines 68 and 99). Search for `FlatBlock::Section {` in the file to find all sites.

```diff
-    FlatBlock::Section { id: "sec1".to_string(), children: vec![], name: None, map_label: None, section_type: None }
+    FlatBlock::Section { id: "sec1".to_string(), children: vec![], name: None, map_label: None, section_type: None, data_file: None, date_prefix: None }
```

Apply the same diff pattern to every other literal construction site for `Section` in the test module.

3. Rewrite `data/sections.yml` with full metadata for all group and section blocks. Source: `69ef14a:data/sections.yml`. Group blocks gain `name:`. Section blocks gain `name:`, `map_label:`, `section_type:`, and where applicable `data_file:` and `date_prefix: true`. Field blocks are unchanged.

New `data/sections.yml`:
```yaml
blocks:
  # Groups
  - type: group
    id: intake
    name: "INTAKE"
    num: 1
    children: [header, adl, exercise, sleep_diet, social, history, specialists]
  - type: group
    id: subjective
    name: "SUBJECTIVE"
    num: 2
    children: [subjective_section]
  - type: group
    id: treatment
    name: "TREATMENT"
    num: 3
    children: [tx_mods, tx_regions]
  - type: group
    id: objective
    name: "OBJECTIVE"
    num: 4
    children: [objective_section]
  - type: group
    id: post_tx
    name: "POST-TX"
    num: 5
    children: [post_treatment, remedial_section, tx_plan, infection_control_section]

  # Sections
  - type: section
    id: header
    name: "Header"
    map_label: "Header"
    section_type: header
    children: [field_date, field_start_time, field_duration, field_appointment_type]
  - type: section
    id: adl
    name: "Activities of Daily Living"
    map_label: "ADL"
    section_type: free_text
  - type: section
    id: exercise
    name: "Exercise Habits"
    map_label: "Exer"
    section_type: free_text
  - type: section
    id: sleep_diet
    name: "Sleep & Diet"
    map_label: "Slp/Dt"
    section_type: free_text
  - type: section
    id: social
    name: "Social & Stress"
    map_label: "Social"
    section_type: free_text
  - type: section
    id: history
    name: "History & Previous Diagnoses"
    map_label: "History"
    section_type: free_text
  - type: section
    id: specialists
    name: "Specialists & Treatment"
    map_label: "Specialists"
    section_type: free_text
  - type: section
    id: subjective_section
    name: "Subjective"
    map_label: "Subjective"
    section_type: free_text
  - type: section
    id: tx_mods
    name: "Treatment Modifications & Preferences"
    map_label: "Tx Mods"
    section_type: list_select
    data_file: "tx_mods.yml"
  - type: section
    id: tx_regions
    name: "Treatment Regions"
    map_label: "Regions"
    section_type: block_select
    data_file: "tx_regions.yml"
  - type: section
    id: objective_section
    name: "Objective / Observations"
    map_label: "Objective"
    section_type: list_select
    data_file: "objective_findings.yml"
    date_prefix: true
  - type: section
    id: post_treatment
    name: "Post-Treatment"
    map_label: "Post-Tx"
    section_type: free_text
  - type: section
    id: remedial_section
    name: "Remedial Exercises & Self-Care"
    map_label: "Remedial"
    section_type: list_select
    data_file: "remedial.yml"
    date_prefix: true
  - type: section
    id: tx_plan
    name: "Treatment Plan / Therapist Notes"
    map_label: "Tx Plan"
    section_type: free_text
  - type: section
    id: infection_control_section
    name: "Infection Control"
    map_label: "Infection Ctrl"
    section_type: checklist
    data_file: "infection_control.yml"

  # Header fields
  - type: field
    id: field_date
  - type: field
    id: field_start_time
  - type: field
    id: field_duration
  - type: field
    id: field_appointment_type
```

4. Rewrite `data/tx_mods.yml` with the full `entries:` list recovered from `69ef14a:data/tx_mods.yml`. The original file used `entries:` with `{ label, output }` pairs - these map directly to `PartOption::Labeled` in the new format under the `entries:` key of the `options-list` block. Retain all 36 entries exactly as they appeared in the original file.

New `data/tx_mods.yml`:
```yaml
blocks:
  - type: options-list
    id: tx_mods_options
    entries:
      - label: "LIGHT PRESSURE: Pt prefers much less than usual pressure."
        output: "- LIGHT PRESSURE: Pt prefers much less than Mr. Gormley's usual working pressure."
      - label: "MODERATE PRESSURE: Pt prefers slightly less than usual pressure."
        output: "- MODERATE PRESSURE: Pt prefers slightly less than Mr. Gormley's usual working pressure."
      - label: "REGULAR PRESSURE: Pt is comfortable with usual working pressure."
        output: "- REGULAR PRESSURE: Pt is comfortable with Mr. Gormley's usual working pressure."
      - label: "FIRM PRESSURE: Pt prefers slightly more than usual pressure."
        output: "- FIRM PRESSURE: Pt prefers slightly more than Mr. Gormley's usual working pressure."
      - label: "HEAVY PRESSURE: Pt prefers much more than usual pressure."
        output: "- HEAVY PRESSURE: Pt prefers much more than Mr. Gormley's usual pressure."
      - label: "FULL PRESSURE: Pt prefers maximum pressure. Consider lowering table."
        output: "- FULL PRESSURE: Pt prefers the maximum pressure that Mr. Gormley can apply safely. Consider lowering the massage table."
      - label: "VERY GENTLE TREATMENT: Avoid all challenging techniques."
        output: "- VERY GENTLE TREATMENT: Pt prefers a gentle treatment: Avoid all challenging techniques."
      - label: "GENTLE TREATMENT: Avoid challenging techniques."
        output: "- GENTLE TREATMENT: Pt prefers a gentle treatment: Avoid challenging techniques."
      - label: "RELAXED TREATMENT: May tolerate some challenge."
        output: "- RELAXED TREATMENT: Pt prefers a relaxed treatment, but may tolerate some challenge."
      - label: "COMFORTABLE TREATMENT: Challenge only when necessary."
        output: "- COMFORTABLE TREATMENT: Pt prefers a challenge only when necessary for treatment."
      - label: "STRONG TREATMENT: Some challenge expected."
        output: "- STRONG TREATMENT: Pt prefers a strong treatment, with some challenge expected."
      - label: "CHALLENGING TREATMENT: Pressure approaching discomfort."
        output: "- CHALLENGING TREATMENT: Pt prefers a challenging treatment, with pressure that approaches discomfort."
      - label: "CALMING: Slow pace, limited conversation."
        output: "- CALMING: Pt responds well to slow-paced techniques with limited conversation.\n- - Give time to breathe and feel muscles relax, without distraction!"
      - label: "RELAXING: Limited or casual conversation."
        output: "- RELAXING: Pt responds well to limited or casual conversation."
      - label: "CONVERSATIONAL: Social atmosphere during treatment."
        output: "- CONVERSATIONAL: Pt responds well to a social atmosphere during treatment.\n- - Remember to interrupt conversation for check ins; allow time for quiet."
      - label: "SOCIAL: Ongoing sociable conversation."
        output: "- SOCIAL: Pt responds well to ongoing sociable conversation during treatment.\n- - If necessary, interrupt conversation to check in!"
      - label: "CONCISE: Clear, direct language, limited conversation."
        output: "- CONCISE: Pt responds well to clear, direct language, with limited conversation."
      - label: "INTROSPECTIVE: Limited conversation, give space for slowing down."
        output: "- INTROSPECTIVE: Pt responds well to limited conversation.\n- - Give space for slowing down without distraction."
      - label: "STOIC: May suppress responses to pain or discomfort."
        output: "- STOIC: Pt may suppress their responses to pain or discomfort. Check in as needed."
      - label: "STOIC: Frequent verbal check ins."
        output: "- STOIC: Pt responds well to frequent verbal check ins.\n- - Pt may suppress their responses to pain or discomfort."
      - label: "CONTROLLED: Check in frequently with each muscle change."
        output: "- CONTROLLED: Pt prefers a very specific amount of pressure.\n- - Check in frequently, even with each muscle change."
      - label: "COMMUNICATIVE: Speaks up about needs, check in regularly."
        output: "- COMMUNICATIVE: Pt speaks up about their needs or discomfort. Check in regularly anyways!"
      - label: "RESTFUL: Limit conversation, focus on recovery."
        output: "- RESTFUL: Pt prefers resting during appointment. Limit conversation, focus on recovery."
      - label: "PREGNANCY: Sidelying, pillowed. Supine up to 5 min. Lower table."
        output: "- PREGNANCY: Patient is treated in sidelying whenever possible, and pillowed under the head, in front of the chest, and between the knees.\n- - Patient may be treated in supine for up to 5 minutes.\n- - Lower table at the beginning and end of treatment."
      - label: "POST-CONCUSSION: Supine only, avoid neck, dim lights, no music."
        output: "- POST-CONCUSSION: Patient is treated supine only, and pillowed under head and knees.\n- - Avoid using prone position.\n- - Avoid the neck.\n- - Dim lighting when possible.\n- - No music.\n- - No air filter fan."
      - label: "VERTIGO: Take time to turn over slowly. Extra time between positions."
        output: "- VERTIGO: Encourage taking time to turn over slowly.\n- - Allow extra time to settle after changing positions before continuing treatment.\n- - Encourage pausing at each step when sitting up from the massage table."
      - label: "LOW LIGHT: Pt prefers lights as dim as possible."
        output: "- LOW LIGHT: Pt prefers as lights as dimmed as possible."
      - label: "NO PRONE: Do not put Pt in prone position."
        output: "- NO PRONE: Do not put Pt in prone position."
      - label: "SUPINE ONLY: Treat Pt only in supine position."
        output: "- SUPINE ONLY: Treat Pt only in supine position."
      - label: "HEAD PILLOW: Pt accepts pillow for head while supine."
        output: "- HEAD PILLOW: Pt accepts pillow for head while supine."
      - label: "HEAD PILLOW REQUIRED: Must be immediately available when supine."
        output: "- HEAD PILLOW REQUIRED: Pillow must be immediately available when supine."
      - label: "LOWER TABLE: Lower massage table before and after treatment."
        output: "- LOWER TABLE: Lower massage table before & after treatment."
      - label: "TALL: Recommend using height extender on massage table."
        output: "- TALL: Recommend using height extender on massage table."
      - label: "CLOTHED: Patient prefers to be clothed during treatment."
        output: "- CLOTHED: Patient prefers to be clothed, using no oil during treatment."
      - label: "RAYNAUD'S: Pt may be sensitive to cold."
        output: "- RAYNAUD'S: Pt may be sensitive to cold."
      - label: "TIMELY: Patient prefers finishing treatment strictly on schedule."
        output: "- TIMELY: Patient prefers finishing treatment strictly on schedule."
```

5. Rewrite `data/objective_findings.yml` with the full `entries:` list recovered from `69ef14a:data/objective_findings.yml` (24 entries, `{ label, output }` pairs).

New `data/objective_findings.yml`:
```yaml
blocks:
  - type: options-list
    id: objective_findings_options
    entries:
      - label: "BL Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "BL Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - label: "BL (L > R) Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "BL (L > R) Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - label: "BL (R > L) Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "BL (R > L) Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - label: "L Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "L Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - label: "R Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "R Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - label: "Rhomboids: Increased Resting Muscle Tension"
        output: "Rhomboids: Increased Resting Muscle Tension"
      - label: "BL Levator Scapula: Increased Resting Muscle Tension"
        output: "BL Levator Scapula: Increased Resting Muscle Tension"
      - label: "BL Erector Spinae: Increased Resting Muscle Tension"
        output: "BL Erector Spinae: Increased Resting Muscle Tension"
      - label: "BL (L > R) Quadratus Lumborum: Tightness & Tender palpated"
        output: "BL (L > R) Quadratus Lumborum: Tightness & Tender palpated"
      - label: "BL Quadratus Lumborum: Increased Resting Muscle Tension"
        output: "BL Quadratus Lumborum: Increased Resting Muscle Tension"
      - label: "BL Piriformis: Increased Resting Muscle Tension"
        output: "BL Piriformis: Increased Resting Muscle Tension"
      - label: "BL Gluteus Medius: Increased Resting Muscle Tension"
        output: "BL Gluteus Medius: Increased Resting Muscle Tension"
      - label: "BL Scalene Muscles: Increased Resting Muscle Tension"
        output: "BL Scalene Muscles: Increased Resting Muscle Tension"
      - label: "BL SCM: Increased Resting Muscle Tension"
        output: "BL Sternocleidomastoid (SCM): Increased Resting Muscle Tension"
      - label: "BL Suboccipital Group: Increased Resting Muscle Tension"
        output: "BL Suboccipital Muscle Group: Increased Resting Muscle Tension"
      - label: "BL Pectoralis Major: Increased Resting Muscle Tension"
        output: "BL Pectoralis Major: Increased Resting Muscle Tension"
      - label: "BL TFL: Increased Resting Muscle Tension"
        output: "BL TFL (Tensor Fasciae Latae): Increased Resting Muscle Tension"
      - label: "BL Hamstrings: Increased Resting Muscle Tension"
        output: "BL Hamstrings: Increased Resting Muscle Tension"
      - label: "BL Gastrocnemius: Increased Resting Muscle Tension"
        output: "BL Gastrocnemius: Increased Resting Muscle Tension"
      - label: "Hypertonicity palpated"
        output: "Hypertonicity palpated"
      - label: "Tightness & Tenderness palpated"
        output: "Tightness & Tender palpated"
      - label: "Trigger points palpated"
        output: "Trigger points palpated"
      - label: "Restricted ROM observed"
        output: "Restricted Range of Motion observed"
      - label: "Protective guarding observed"
        output: "Protective guarding observed"
```

6. Rewrite `data/remedial.yml` with the full `entries:` list recovered from `69ef14a:data/remedial.yml` (14 entries).

New `data/remedial.yml`:
```yaml
blocks:
  - type: options-list
    id: remedial_options
    entries:
      - label: "PSOAS KNEELING STRETCH - Low Back Pain"
        output: "PSOAS KNEELING STRETCH - Lengthen Psoas to Reduce Low Back Pain:\n- - \"Slide hips all the way forwards, then rotate all the way over the top knee.\n- - Slight lean back, and disco point to the back corner.\"\n- - 60s static stretch, bilaterally (30s-120s).\n- - 5 / week, for 2 weeks or until next appointment."
      - label: "WALL ANGEL - Rhomboid Strength & Postural Awareness"
        output: "WALL ANGEL - Increase Rhomboid Strength & Postural Awareness:\n- - 5->10 Reps, daily at work, for 3 weeks or until next appointment.\n- - \"Perfect reps, no rushing!\""
      - label: "CACTUS ARMS (Angel Wings) - Rhomboid Strength"
        output: "CACTUS ARMS (Angel Wings) - Increase Rhomboid Strength & Postural Awareness:\n- - 5 Reps, daily at work, for 3 weeks or until next appointment."
      - label: "SPHINX STRETCH - Lumbar Mobility"
        output: "SPHINX STRETCH / HOLDS - Improve Lumbar Mobility:\n- - 8-12 Reps (hold 5s) x 1 Set.\n- - 3 / week, for 2 weeks or until next appointment."
      - label: "RECTUS FEMORIS KNEELING WALL STRETCH - Knee and Calf Tightness"
        output: "RECTUS FEMORIS KNEELING WALL STRETCH - Lengthen Rec Fem to Reduce Knee and Calf Tightness:\n- - \"Kneel against the base of the wall, then rotate one foot up.\n- - Keeping the foot against the wall, slowly raise your torso vertically to stretch.\"\n- - 60s static stretch, bilaterally (30s-120s).\n- - 2-4 / week, for 6 weeks or until next appointment."
      - label: "RAINBOW ARC SCM STRETCH - Lengthen SCM"
        output: "RAINBOW ARC SCM STRETCH - Lengthen Sternocleidomastoid (SCM):\n- - \"Laterally flex your neck, then make an arc with your nose from front, upwards, and back.\n- - To intensify the stretch, reach down with one arm.\"\n- - 30s-60s dynamic stretch, bilaterally (60s-120s total).\n- - 1-2 daily, or as needed."
      - label: "GLUTE BRIDGES - Gluteal Strength, Core Stability"
        output: "GLUTE BRIDGES - Increase Gluteal Strength, Core Stability:\n- - 12 Reps (hold 3 sec) x 3 Sets (Rest 60s).\n- - \"Squeeze the glutes!\"\n- - 3 / week, for 6 weeks or until next appointment."
      - label: "BODYWEIGHT SQUATS - Gluteal Strength"
        output: "BODYWEIGHT SQUATS - Increase Gluteal Strength:\n- - 8->12 Reps x 3 Sets (Rest 60s), using 80% strength.\n- - 2 / week, for 6 weeks or until next appointment."
      - label: "BENT-OVER DUMBBELL ROW - Rhomboid Strength"
        output: "BENT-OVER DUMBBELL ROW - Increase Rhomboid Strength:\n- - 8-10 Reps x 3 Sets (Rest 60s), using 80% strength.\n- - 4 / week, for 3 weeks or until next appointment."
      - label: "REGULAR WALKS - Daily Physical Activity"
        output: "REGULAR WALKS - Increase Daily Physical Activity:\n- - As early in the day as possible, go outside for a daily walk.\n- - Even 5 minutes is a great start! Do longer if you can, and avoid skipping entirely."
      - label: "Recommend returning to regular exercise habits."
        output: "Recommend returning to regular exercise habits."
      - label: "Continue exercise program as directed by physiotherapist."
        output: "Continue exercise program as directed by physiotherapist."
      - label: "Continue exercise program as directed by chiropractor."
        output: "Continue exercise program as directed by chiropractor."
      - label: "No specific exercise or self-care goals discussed."
        output: "No specific exercise or self-care goals discussed."
```

7. Rewrite `data/infection_control.yml` with entries recovered from `69ef14a:data/infection_control.yml`. The original used an `items:` list of plain strings. In the flat format these become `{ label: "...", output: "..." }` pairs where label and output are identical (checklist items have no separate display/output distinction).

New `data/infection_control.yml`:
```yaml
blocks:
  - type: options-list
    id: infection_control_options
    entries:
      - label: "Disinfected patient contact points before and after appointment (door knob, coat hooks, patient chair)."
        output: "Disinfected patient contact points before and after appointment (door knob, coat hooks, patient chair)."
      - label: "Disinfected equipment before and after appointment (massage table, face cradle, RMT rolling stool, oil bottle)."
        output: "Disinfected equipment before and after appointment (massage table, face cradle, RMT rolling stool, oil bottle)."
      - label: "Clean Linens used (fitted sheet, flat sheet, face cradle cover, blanket)."
        output: "Clean Linens used (fitted sheet, flat sheet, face cradle cover, blanket)."
      - label: "RMT washed hands before and after treatment."
        output: "RMT washed hands before and after treatment."
      - label: "Mask worn by RMT."
        output: "Mask worn by RMT."
```

8. Rewrite `data/muscles.yml` with the full `entries:` list recovered from `f664b7f:data/muscles.yml` (51 entries, all `{ label, output }` pairs).

New `data/muscles.yml`:
```yaml
blocks:
  - type: options-list
    id: muscles_options
    entries:
      - label: "Masseter (TMJ)"
        output: "Masseter"
      - label: "Temporalis (TMJ)"
        output: "Temporalis"
      - label: "Suprahyoid Muscles (TMJ)"
        output: "Suprahyoid muscles"
      - label: "Lateral Pterygoid (TMJ)"
        output: "Lateral Pterygoid"
      - label: "Medial Pterygoid (TMJ)"
        output: "Medial Pterygoid"
      - label: "Sternocleidomastoid (SCM)"
        output: "Sternocleidomastoid (SCM)"
      - label: "Suboccipital Muscle Group"
        output: "Suboccipital Muscle Group"
      - label: "Scalene Muscles"
        output: "Scalene Muscles"
      - label: "Splenius Capitis"
        output: "Splenius Capitis"
      - label: "Splenius Cervicis"
        output: "Splenius Cervicis"
      - label: "Pectoralis Major"
        output: "Pectoralis Major"
      - label: "Pectoralis Minor"
        output: "Pectoralis Minor"
      - label: "Serratus Anterior"
        output: "Serratus Anterior"
      - label: "Intercostal Muscles"
        output: "Intercostal Muscles"
      - label: "Psoas Major"
        output: "Psoas Major"
      - label: "Iliacus"
        output: "Iliacus"
      - label: "Trapezius (Upper Fibers)"
        output: "Trapezius (Upper Fibers)"
      - label: "Trapezius (Middle Fibers)"
        output: "Trapezius (Middle Fibers)"
      - label: "Trapezius (Lower Fibers)"
        output: "Trapezius (Lower Fibers)"
      - label: "Levator Scapula"
        output: "Levator Scapula"
      - label: "Rhomboids (Major & Minor)"
        output: "Rhomboids"
      - label: "Teres Major"
        output: "Teres Major"
      - label: "Teres Minor"
        output: "Teres Minor"
      - label: "Subscapularis"
        output: "Subscapularis"
      - label: "Infraspinatus"
        output: "Infraspinatus"
      - label: "Supraspinatus"
        output: "Supraspinatus"
      - label: "Erector Spinae"
        output: "Erector Spinae"
      - label: "Quadratus Lumborum (QL)"
        output: "Quadratus Lumborum"
      - label: "Multifidus"
        output: "Multifidus"
      - label: "Piriformis"
        output: "Piriformis"
      - label: "Gluteus Maximus"
        output: "Gluteus Maximus"
      - label: "Gluteus Medius"
        output: "Gluteus Medius"
      - label: "Gluteus Minimus"
        output: "Gluteus Minimus"
      - label: "TFL (Tensor Fasciae Latae)"
        output: "TFL (Tensor Fasciae Latae)"
      - label: "Biceps Femoris (Hamstring)"
        output: "Biceps Femoris"
      - label: "Semitendinosus (Hamstring)"
        output: "Semitendinosus"
      - label: "Semimembranosus (Hamstring)"
        output: "Semimembranosus"
      - label: "Rectus Femoris (Quad)"
        output: "Rectus Femoris"
      - label: "Vastus Lateralis (Quad)"
        output: "Vastus Lateralis"
      - label: "Vastus Medialis (Quad)"
        output: "Vastus Medialis"
      - label: "Gastrocnemius"
        output: "Gastrocnemius"
      - label: "Soleus"
        output: "Soleus"
      - label: "Tibialis Anterior"
        output: "Tibialis Anterior"
      - label: "Biceps Brachii"
        output: "Biceps Brachii"
      - label: "Brachialis"
        output: "Brachialis"
      - label: "Triceps Brachii"
        output: "Triceps Brachii"
      - label: "Extensor Digitorum"
        output: "Extensor Digitorum"
      - label: "Extensor Carpi Ulnaris"
        output: "Extensor Carpi Ulnaris"
      - label: "Flexor Carpi Radialis"
        output: "Flexor Carpi Radialis"
      - label: "Deltoid (Anterior)"
        output: "Deltoid (Anterior)"
      - label: "Deltoid (Posterior)"
        output: "Deltoid (Posterior)"
```

9. Run the full test suite to confirm no regressions: `cargo test`. The `real_data_dir_loads_as_flat_format` test and all flat_file tests must still pass. If the loader performs any validation on `data_file` references, it will not fail because `load_data_dir` currently skips unrecognized extra fields on deserialization (they are `Option<...>` with `#[serde(default)]` and the loader only checks `children:` references, not `data_file:` values).

## Verification

### Manual tests
- Run `cargo run` and verify the app launches without panics.
- Navigate to the TREATMENT group and confirm the Tx Mods section shows the full pressure/style options list.
- Navigate to the OBJECTIVE group and confirm the Objective section shows the muscle findings list.
- Navigate to POST-TX and confirm Remedial and Infection Control sections show their respective entries.
- Confirm the INTAKE group map shows correct labels (ADL, Exer, Slp/Dt, Social, History, Specialists).

### Automated tests
- `cargo test` - all existing tests (currently passing) must continue to pass after step 1 extends `FlatBlock::Section`. Specifically `flat_file::tests::flat_block_section_variant_has_id`, `flat_file::tests::section_block_deserializes_name_map_label_section_type`, and `real_data_dir_loads_as_flat_format` must remain green.
- `cargo build` - must produce no new compiler errors or warnings.

## Changelog

### Review - 2026-04-01
- #1: Fixed entry count for tx_mods in Critical Files and Step 4: 34 -> 36 (verified against 69ef14a)
- #2: Fixed entry count for muscles in Critical Files and Step 8: 46 -> 51 (verified against f664b7f)
- #3: Fixed Critical Files line reference for Section test literals: line 83 -> 99-100 (line 83 is OptionsList test, not Section)

### Review #2 - 2026-04-01
- #4: Fixed Step 2 literal count: "four" -> "two" and removed incorrect line 84 reference (line 84 is OptionsList test, not Section); confirmed only 2 Section literal construction sites at lines 68 and 99
- #5: Fixed Reuse section PartOption::Labeled line reference: line 12 -> line 15 (line 12 is the enum declaration; Labeled variant is at line 15)

### Prefect-1 – 2026-04-01
- nit-1: Fixed Critical Files Section variant line range: 17-23 -> 16-22 (Section { opens at line 16, closing }, at line 22; verified against actual flat_file.rs)

## Prefect-1 Report

**Round:** 1
**Verdict:** Nit only — one line-range inaccuracy in Critical Files.

| # | Severity | Location | Issue | Fix Applied |
|---|----------|----------|-------|-------------|
| nit-1 | nit | Critical Files, line 13 | Section variant line range stated as 17-23; actual span is 16-22 (Section { at line 16, closing }, at line 22) | Corrected to 16-22 |

All diffs apply cleanly against the actual source. Entry counts verified (tx_mods=36, objective_findings=24, remedial=14, infection_control=5, muscles=51). Step 2 literals match actual lines 68 and 99. PartOption::Labeled at line 15 confirmed correct.

## Progress
- Step 1: Added `data_file: Option<String>` and `date_prefix: Option<bool>` to `FlatBlock::Section` in src/flat_file.rs
- Step 2: Updated both `FlatBlock::Section` literal construction sites in tests (lines 68 and 99) with new fields set to None
- Step 3: Rewrote data/sections.yml with full group names/nums and section name/map_label/section_type/data_file/date_prefix fields
- Step 4: Rewrote data/tx_mods.yml with all 36 entries restored
- Step 5: Rewrote data/objective_findings.yml with all 24 entries restored
- Step 6: Rewrote data/remedial.yml with all 14 entries restored
- Step 7: Rewrote data/infection_control.yml with all 5 entries restored
- Step 8: Rewrote data/muscles.yml with all 51 entries restored
- Step 9: cargo test - 66 tests passed, 0 failed

## Implementation
Complete - 2026-04-01
