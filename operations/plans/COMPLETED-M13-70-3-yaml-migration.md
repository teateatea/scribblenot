## Task

#70 - Implement canonical 6-level YAML data hierarchy

## Context

Sub-tasks 1 and 2 added the hierarchy structs and `load_hierarchy_dir`. Sub-task 3 migrates all 6 data YAML files (`sections.yml`, `tx_regions.yml`, `boilerplate.yml`, `objective_findings.yml`, `remedial.yml`, `infection_control.yml`) from the old flat/block formats to `HierarchyFile` format. After this sub-task, `load_hierarchy_dir` must be able to parse the real `data/` directory and return `Ok`. The mission test criterion "No map_label: keys survive in any data/*.yml file" and "All data YAML files parse as HierarchyFile without error" must both be satisfied.

Two struct-level prerequisites must be resolved first: `HierarchySection` currently requires both `nav_label` and `map_label` as mandatory `String` fields, and lacks `data_file`, `heading_search_text`, `heading_label`, `note_render_slot`, and `is_intake`. The migrated YAML must use only `nav_label` (no `map_label:` keys), and the shim in ST4 must be able to read `data_file` and the metadata fields from parsed `HierarchySection` structs. These gaps must be closed in a prerequisite step before the YAML files are written.

## Approach

Step 1 extends `HierarchySection` in `src/data.rs` to add the missing fields and make `map_label` optional with a default empty string (so YAML files that omit it still parse). Step 2 migrates `sections.yml`. Step 3 migrates `tx_regions.yml`. Step 4 migrates `boilerplate.yml`. Step 5 migrates the three list-select / checklist files (`objective_findings.yml`, `remedial.yml`, `infection_control.yml`). Step 6 verifies all files parse without error.

The `num:` field on groups in sections.yml is a legacy carry-through. `HierarchyGroup` currently has no `num` field - the same prerequisite step adds `num: Option<usize>` to `HierarchyGroup`. serde_yaml silently ignores unknown fields by default, so unknown fields in YAML do not cause parse errors; however every field the shim needs in ST4 must be present on the struct.

## Critical Files

- `src/data.rs` lines 650-666 - `HierarchySection` and `HierarchyGroup` struct definitions to extend
- `/c/scribble/data/sections.yml` - full replacement
- `/c/scribble/data/tx_regions.yml` - full replacement
- `/c/scribble/data/boilerplate.yml` - full replacement
- `/c/scribble/data/objective_findings.yml` - full replacement
- `/c/scribble/data/remedial.yml` - full replacement
- `/c/scribble/data/infection_control.yml` - full replacement

## Reuse

- `load_hierarchy_dir` from sub-task 2 - used in Step 6 verification to confirm all 6 files parse
- Existing `serde_yaml::from_str` pattern - unchanged
- `#[serde(default)]` attribute - used on `map_label` to make it optional in YAML without breaking existing tests that set it

## Steps

### Step 1: Extend HierarchySection and HierarchyGroup

In `src/data.rs`, extend `HierarchySection` to add the missing shim-required fields and make `map_label` optional:

```
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct HierarchySection {
     pub id: String,
     pub nav_label: String,
-    pub map_label: String,
+    #[serde(default)]
+    pub map_label: String,
     pub section_type: String,
     pub fields: Option<Vec<HierarchyField>>,
     pub lists: Option<Vec<HierarchyList>>,
     pub date_prefix: Option<bool>,
+    pub data_file: Option<String>,
+    pub heading_search_text: Option<String>,
+    pub heading_label: Option<String>,
+    pub note_render_slot: Option<String>,
+    #[serde(default)]
+    pub is_intake: bool,
 }
```

Extend `HierarchyGroup` to add the `num` legacy field:

```
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct HierarchyGroup {
     pub id: String,
     pub nav_label: String,
     pub sections: Vec<String>,
+    pub num: Option<usize>,
 }
```

Also extend `HierarchyField` to carry the `composite`, `default`, and `repeat_limit` fields that the new sections.yml will include (these are silently ignored by serde today but must be present on the struct before ST4 wires the shim):

```
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct HierarchyField {
     pub id: String,
     pub label: String,
     pub field_type: String,
     #[serde(default)]
     pub options: Vec<String>,
     pub list_id: Option<String>,
     pub data_file: Option<String>,
+    pub composite: Option<CompositeConfig>,
+    pub default: Option<String>,
+    pub repeat_limit: Option<usize>,
 }
```

After this change, run `cargo test` to confirm the 10 `hierarchy_struct_tests` still pass and there are no regressions. The existing TEST-6 in `hierarchy_struct_tests` sets `map_label: "SEC 1"` in the YAML fixture and asserts `section.map_label == "SEC 1"` - this continues to pass because the field is still present, just now `#[serde(default)]` lets it be omitted.

### Step 2: Migrate sections.yml

Replace the entire contents of `/c/scribble/data/sections.yml` with the following:

```yaml
template:
  groups:
    - intake
    - subjective
    - treatment
    - objective
    - post_tx

groups:
  - id: intake
    nav_label: "INTAKE"
    num: 1
    sections:
      - header
      - adl
      - exercise
      - sleep_diet
      - social
      - history
      - specialists

  - id: subjective
    nav_label: "SUBJECTIVE"
    num: 2
    sections:
      - subjective_section

  - id: treatment
    nav_label: "TREATMENT"
    num: 3
    sections:
      - tx_mods
      - tx_regions

  - id: objective
    nav_label: "OBJECTIVE"
    num: 4
    sections:
      - objective_section

  - id: post_tx
    nav_label: "POST-TX"
    num: 5
    sections:
      - post_treatment
      - remedial_section
      - tx_plan
      - infection_control_section

sections:
  - id: header
    nav_label: "Header"
    section_type: multi_field
    note_render_slot: "header"
    fields:
      - id: date
        label: "Date"
        field_type: composite
        composite:
          format: "{year}-{month}-{day}"
          parts:
            - id: day
              label: "Day"
              preview: "DD"
              sticky: true
              options: ["01","02","03","04","05","06","07","08","09","10","11","12","13","14","15","16","17","18","19","20","21","22","23","24","25","26","27","28","29","30","31"]
            - id: month
              label: "Month"
              preview: "MM"
              sticky: true
              options: ["01","02","03","04","05","06","07","08","09","10","11","12"]
            - id: year
              label: "Year"
              preview: "YYYY"
              sticky: true
              options: ["2025","2026"]
      - id: start_time
        label: "Start Time"
        field_type: composite
        composite:
          format: "{hour}:{minute}"
          parts:
            - id: hour
              label: "Hour"
              preview: "H"
              default: "1"
              options: ["9","10","11","12","1","2","3","4","5","6","7","8"]
            - id: minute
              label: "Minute"
              preview: "MM"
              default: 00-2
              options:
                - id: 00-1
                  label: "00"
                  output: "00"
                - id: 15-1
                  label: "15"
                  output: "15"
                - id: 30-1
                  label: "30"
                  output: "30"
                - id: 45-1
                  label: "45"
                  output: "45"
                - id: 00-2
                  label: "00"
                  output: "00"
                - id: 15-2
                  label: "15"
                  output: "15"
                - id: 30-2
                  label: "30"
                  output: "30"
                - id: 45-2
                  label: "45"
                  output: "45"
                - id: 00-3
                  label: "00"
                  output: "00"
      - id: appointment_duration
        label: "Duration (min)"
        field_type: select
        options: ["30","45","60","75","90"]
        default: "60"
      - id: appointment_type
        label: "Appointment Type"
        field_type: composite
        composite:
          format: "{treatment}"
          parts:
            - id: treatment
              label: "Appointment Type"
              default: "Treatment"
              options:
                - label: "Treatment"
                  output: "Treatment focused massage"
                - label: "Relaxation"
                  output: "Relaxation"
                - label: "General"
                  output: "General massage"

  - id: adl
    nav_label: "ADL"
    section_type: free_text
    is_intake: true
    heading_label: "#### ACTIVITIES OF DAILY LIVING"
    heading_search_text: "ACTIVITIES OF DAILY LIVING"

  - id: exercise
    nav_label: "Exer"
    section_type: free_text
    is_intake: true
    heading_label: "#### EXERCISE HABITS"
    heading_search_text: "EXERCISE HABITS"

  - id: sleep_diet
    nav_label: "Slp/Dt"
    section_type: free_text
    is_intake: true
    heading_label: "#### SLEEP & DIET"
    heading_search_text: "SLEEP & DIET"

  - id: social
    nav_label: "Social"
    section_type: free_text
    is_intake: true
    heading_label: "#### SOCIAL & STRESS"
    heading_search_text: "SOCIAL & STRESS"

  - id: history
    nav_label: "History"
    section_type: free_text
    is_intake: true
    heading_label: "#### HISTORY & PREVIOUS DIAGNOSES"
    heading_search_text: "HISTORY & PREVIOUS DIAGNOSES"

  - id: specialists
    nav_label: "Specialists"
    section_type: free_text
    is_intake: true
    heading_label: "#### SPECIALISTS & TREATMENT"
    heading_search_text: "SPECIALISTS & TREATMENT"

  - id: subjective_section
    nav_label: "Subjective"
    section_type: free_text
    heading_search_text: "## SUBJECTIVE"
    note_render_slot: "subjective_section"

  - id: tx_mods
    nav_label: "Tx Mods"
    section_type: multi_field
    heading_search_text: "TREATMENT MODIFICATIONS"
    note_render_slot: "tx_mods"
    fields:
      - id: pressure
        label: "Pressure"
        field_type: select
        options:
          - "- LIGHT PRESSURE: Pt prefers much less than Mr. Gormley's usual working pressure."
          - "- MODERATE PRESSURE: Pt prefers slightly less than Mr. Gormley's usual working pressure."
          - "- REGULAR PRESSURE: Pt is comfortable with Mr. Gormley's usual working pressure."
          - "- FIRM PRESSURE: Pt prefers slightly more than Mr. Gormley's usual working pressure."
          - "- HEAVY PRESSURE: Pt prefers much more than Mr. Gormley's usual pressure."
          - "- FULL PRESSURE: Pt prefers the maximum pressure that Mr. Gormley can apply safely. Consider lowering the massage table."
      - id: challenge
        label: "Challenge"
        field_type: select
        options:
          - "- VERY GENTLE TREATMENT: Pt prefers a gentle treatment: Avoid all challenging techniques."
          - "- GENTLE TREATMENT: Pt prefers a gentle treatment: Avoid challenging techniques."
          - "- RELAXED TREATMENT: Pt prefers a relaxed treatment, but may tolerate some challenge."
          - "- COMFORTABLE TREATMENT: Pt prefers a challenge only when necessary for treatment."
          - "- STRONG TREATMENT: Pt prefers a strong treatment, with some challenge expected."
          - "- CHALLENGING TREATMENT: Pt prefers a challenging treatment, with pressure that approaches discomfort."
      - id: mood
        label: "Mood"
        field_type: select
        options:
          - "- CALMING: Pt responds well to slow-paced techniques with limited conversation.\n- - Give time to breathe and feel muscles relax, without distraction!"
          - "- RELAXING: Pt responds well to limited or casual conversation."
          - "- CONVERSATIONAL: Pt responds well to a social atmosphere during treatment.\n- - Remember to interrupt conversation for check ins; allow time for quiet."
          - "- SOCIAL: Pt responds well to ongoing sociable conversation during treatment.\n- - If necessary, interrupt conversation to check in!"
      - id: communication
        label: "Communication"
        field_type: select
        options:
          - "- CONCISE: Pt responds well to clear, direct language, with limited conversation."
          - "- INTROSPECTIVE: Pt responds well to limited conversation.\n- - Give space for slowing down without distraction."
          - "- STOIC: Pt may suppress their responses to pain or discomfort. Check in as needed."
          - "- STOIC: Pt responds well to frequent verbal check ins.\n- - Pt may suppress their responses to pain or discomfort."
          - "- CONTROLLED: Pt prefers a very specific amount of pressure.\n- - Check in frequently, even with each muscle change."
          - "- COMMUNICATIVE: Pt speaks up about their needs or discomfort. Check in regularly anyways!"
          - "- RESTFUL: Pt prefers resting during appointment. Limit conversation, focus on recovery."
      - id: modifications
        label: "Modifications"
        field_type: select
        repeat_limit: 10
        options:
          - "- PREGNANCY: Patient is treated in sidelying whenever possible, and pillowed under the head, in front of the chest, and between the knees.\n- - Patient may be treated in supine for up to 5 minutes.\n- - Lower table at the beginning and end of treatment."
          - "- POST-CONCUSSION: Patient is treated supine only, and pillowed under head and knees.\n- - Avoid using prone position.\n- - Avoid the neck.\n- - Dim lighting when possible.\n- - No music.\n- - No air filter fan."
          - "- VERTIGO: Encourage taking time to turn over slowly.\n- - Allow extra time to settle after changing positions before continuing treatment.\n- - Encourage pausing at each step when sitting up from the massage table."
          - "- LOW LIGHT: Pt prefers as lights as dimmed as possible."
          - "- NO PRONE: Do not put Pt in prone position."
          - "- SUPINE ONLY: Treat Pt only in supine position."
          - "- HEAD PILLOW: Pt accepts pillow for head while supine."
          - "- HEAD PILLOW REQUIRED: Pillow must be immediately available when supine."
          - "- LOWER TABLE: Lower massage table before & after treatment."
          - "- TALL: Recommend using height extender on massage table."
          - "- CLOTHED: Patient prefers to be clothed, using no oil during treatment."
          - "- RAYNAUD'S: Pt may be sensitive to cold."
          - "- TIMELY: Patient prefers finishing treatment strictly on schedule."

  - id: tx_regions
    nav_label: "Regions"
    section_type: block_select
    heading_search_text: "TREATMENT / PLAN"
    note_render_slot: "tx_regions"

  - id: objective_section
    nav_label: "Objective"
    section_type: list_select
    data_file: "objective_findings.yml"
    date_prefix: true
    heading_search_text: "## OBJECTIVE / OBSERVATIONS"
    note_render_slot: "objective_section"

  - id: post_treatment
    nav_label: "Post-Tx"
    section_type: free_text
    heading_search_text: "## POST-TREATMENT"
    note_render_slot: "post_treatment"

  - id: remedial_section
    nav_label: "Remedial"
    section_type: list_select
    data_file: "remedial.yml"
    date_prefix: true
    heading_search_text: "REMEDIAL EXERCISES"
    note_render_slot: "remedial_section"

  - id: tx_plan
    nav_label: "Tx Plan"
    section_type: free_text
    heading_search_text: "TREATMENT PLAN"
    note_render_slot: "tx_plan"

  - id: infection_control_section
    nav_label: "Infection Ctrl"
    section_type: checklist
    data_file: "infection_control.yml"
    heading_search_text: "INFECTION CONTROL"
    note_render_slot: "infection_control_section"
```

Notes on sections.yml migration:
- `type: group/section/field` discriminants are gone; hierarchy is conveyed by top-level YAML keys.
- `name:` renames to `nav_label:` on groups and sections. `map_label:` keys are completely removed from all sections in sections.yml (Key Decision 3; BRIEF manual test criterion: `grep -r "map_label:" /c/scribble/data/` must return zero results). The shim in ST4 derives `SectionConfig.map_label` from `section.nav_label` directly. The `#[serde(default)]` on `HierarchySection.map_label` (added in Step 1) lets the field be absent in YAML, defaulting to empty string, until the shim overwrites it from `nav_label`.
- `children:` on groups becomes `sections: [list of section IDs]`.
- `children:` on sections becomes inline `fields:`.
- `is_intake: true` is kept on the 6 intake sections; the shim copies it into `SectionConfig.is_intake`.
- The `tx_regions` section has no `data_file:` and no inline `lists:` in sections.yml. tx_regions.yml carries all the lists; the shim in ST4 will resolve them via `section.id == "tx_regions"` lookup from the merged `lists` pool (not via `data_file` dispatch). This is consistent with Key Decision 5 and 6 in APPROVED-70-yaml-data-hierarchy.md.
- `HierarchyField` is extended in Step 1 to carry `composite: Option<CompositeConfig>`, `default: Option<String>`, and `repeat_limit: Option<usize>` so the inline `composite:` and `repeat_limit:` YAML keys shown above parse into the struct rather than being silently ignored. ST4's shim reads `HeaderFieldConfig` fields from the old flat path; these additions do not affect ST3 runtime behavior but keep the struct complete before ST4 arrives.

### Step 3: Migrate tx_regions.yml

Replace the entire contents of `/c/scribble/data/tx_regions.yml` with the following. The file contains only `lists:` - no `sections:` entry (Key Decision 6: section definition stays in sections.yml to avoid duplicate Section ID).

```yaml
lists:
  - id: back_all_prone
    label: "UPPER, MIDDLE & LOWER BACK (Prone)"
    items:
      - id: swedish
        label: "General Swedish Techniques"
        output: "- General Swedish Techniques"
      - id: spec_comp_back
        label: "Specific Compressions (Trapezius, LS, Teres, QL)"
        output: "- Specific Compressions:\n- - Trapezius (Upper Fiber)\n- - Levator Scapula\n- - Teres Major & Minor\n- - Quadratus Lumborum"
      - id: stretch_sa
        label: "Stretch (Serratus Anterior)"
        output: "- Stretch (Serratus Anterior)"
      - id: broad_comp_triceps
        label: "Broad Compressions (Triceps Brachii)"
        output: "- Broad Compressions (Triceps Brachii)"
      - id: muscle_strip_es
        label: "Muscle Stripping (Erector Spinae)"
        output: "- Muscle Stripping (Erector Spinae)"

  - id: back_upper_mid_prone
    label: "UPPER & MIDDLE BACK (Prone)"
    items:
      - id: swedish
        label: "General Swedish Techniques"
        output: "- General Swedish Techniques"
      - id: spec_comp_back
        label: "Specific Compressions (Trapezius, Teres)"
        output: "- Specific Compressions:\n- - Trapezius (Upper Fiber)\n- - Teres Major & Minor"
      - id: muscle_strip_es
        label: "Muscle Stripping (Erector Spinae)"
        output: "- Muscle Stripping (Erector Spinae)"

  - id: back_lower_prone
    label: "LOWER BACK (Prone)"
    items:
      - id: swedish
        label: "General Swedish Techniques"
        output: "- General Swedish Techniques"
      - id: spec_comp_ql
        label: "Specific Compressions (Quadratus Lumborum)"
        output: "- Specific Compressions (Quadratus Lumborum)"
      - id: muscle_strip_es
        label: "Muscle Stripping (Erector Spinae)"
        output: "- Muscle Stripping (Erector Spinae)"
      - id: fascial_l4l5
        label: "Fascial Techniques (L4-L5)"
        output: "- Fascial Techniques (L4-L5)"
        default: false

  - id: glutes_prone
    label: "GLUTEAL MUSCLES / BUTTOCKS (Prone)"
    items:
      - id: broad_comp
        label: "Broad Compressions"
        output: "- Broad Compressions"
      - id: ulnar_knead
        label: "Ulnar Kneading"
        output: "- Ulnar Kneading"
      - id: spec_comp_pir_gmed
        label: "Specific Compressions (Piriformis, Gluteus Medius)"
        output: "- Specific Compressions (Ulnar border):\n- - Piriformis\n- - Gluteus Medius"
      - id: pin_stretch_pir
        label: "Pin + Stretch (Piriformis)"
        output: "- Pin + Stretch:\n- - Piriformis"
      - id: rocking_si
        label: "Rocking & Rhythmic Techniques (SI Joint)"
        output: "- Rocking & Rhythmic Techniques (SI Jt)"

  - id: post_legs_prone
    label: "POSTERIOR LEGS & FEET (Prone)"
    items:
      - id: broad_comp
        label: "Broad Compressions"
        output: "- Broad Compressions"
      - id: ulnar_knead_ham
        label: "Ulnar Kneading (Biceps Femoris, Semitendinosus)"
        output: "- Ulnar Kneading\n- - Biceps Femoris\n- - Semitendinosus"
      - id: knuckle_knead
        label: "Knuckle Kneading"
        output: "- Knuckle Kneading"
      - id: fingertip_knead
        label: "Fingertip Kneading"
        output: "- Fingertip Kneading"

  - id: post_upper_legs_prone
    label: "POSTERIOR UPPER LEGS / HAMSTRINGS (Prone)"
    items:
      - id: broad_comp
        label: "Broad Compressions"
        output: "- Broad Compressions"
      - id: ulnar_knead_ham
        label: "Ulnar Kneading (Biceps Femoris, Semitendinosus)"
        output: "- Ulnar Kneading\n- - Biceps Femoris\n- - Semitendinosus"

  - id: lower_legs_prone
    label: "LOWER LEGS & FEET (Prone)"
    items:
      - id: broad_comp
        label: "Broad Compressions"
        output: "- Broad Compressions"
      - id: ulnar_knead
        label: "Ulnar Kneading"
        output: "- Ulnar Kneading"
      - id: knuckle_knead
        label: "Knuckle Kneading"
        output: "- Knuckle Kneading"
      - id: fingertip_knead
        label: "Fingertip Kneading"
        output: "- Fingertip Kneading"

  - id: ant_legs_hips_supine
    label: "ANTERIOR LEGS & HIPS (Supine)"
    items:
      - id: broad_comp
        label: "Broad Compressions"
        output: "- Broad Compressions"
      - id: ulnar_knead_tfl
        label: "Ulnar Kneading (TFL, Vastus Lateralis & Medialis)"
        output: "- Ulnar Kneading (TFL, Vastus Lateralis & Medialis)"
      - id: fingertip_knead
        label: "Fingertip Kneading"
        output: "- Fingertip Kneading"
      - id: long_axis_traction
        label: "Long Axis Traction"
        output: "- Long Axis Traction"

  - id: ant_hips_upper_legs_supine
    label: "ANTERIOR HIPS & UPPER LEGS / TFL & QUADS (Supine)"
    items:
      - id: broad_comp
        label: "Broad Compressions"
        output: "- Broad Compressions"
      - id: ulnar_knead_tfl
        label: "Ulnar Kneading (TFL, Vastus Lateralis & Medialis)"
        output: "- Ulnar Kneading (TFL, Vastus Lateralis & Medialis)"
      - id: spec_comp_tfl
        label: "Specific Compressions (TFL, Vastus Lateralis & Medialis)"
        output: "- Specific Compressions (TFL, Vastus Lateralis & Medialis)"
      - id: friction_rf
        label: "Friction (Rectus Femoris)"
        output: "- Friction (Rectus Femoris)"
      - id: jt_mob_hip
        label: "Joint Mobilization (Hip: Traction, Grade 2)"
        output: "- Joint Mobilization (Coxafemoral Jt: Traction, Grade 2)"

  - id: lower_legs_supine
    label: "LOWER LEGS & FEET (Supine)"
    items:
      - id: broad_comp
        label: "Broad Compressions"
        output: "- Broad Compressions"
      - id: ulnar_knead
        label: "Ulnar Kneading"
        output: "- Ulnar Kneading"
      - id: knuckle_knead
        label: "Knuckle Kneading"
        output: "- Knuckle Kneading"
      - id: fingertip_knead
        label: "Fingertip Kneading"
        output: "- Fingertip Kneading"

  - id: abdomen_intercostals_supine
    label: "ABDOMEN - INTERCOSTALS (Supine)"
    items:
      - id: myofascial
        label: "Myofascial Techniques"
        output: "- Myofascial Techniques"
      - id: spec_comp
        label: "Specific Compressions"
        output: "- Specific Compressions"
      - id: fingertip_knead_intercostal
        label: "Fingertip Kneading (Intercostal Muscles)"
        output: "- Fingertip Kneading (Intercostal Muscles)"

  - id: abdomen_psoas_supine
    label: "ABDOMEN - PSOAS (Supine)"
    items:
      - id: myofascial
        label: "Myofascial Techniques"
        output: "- Myofascial Techniques"
      - id: spec_comp_psoas
        label: "Specific Compressions (Psoas Major, Iliacus)"
        output: "- Specific Compressions (Psoas Major, Iliacus)"
      - id: pin_active_psoas
        label: "Pin & Active Release (Psoas)"
        output: "- Pin & Active Release (Psoas)"

  - id: adductors_sidelying
    label: "ADDUCTORS (Sidelying)"
    items:
      - id: broad_comp
        label: "Broad Compressions"
        output: "- Broad Compressions"
      - id: ulnar_knead
        label: "Ulnar Kneading"
        output: "- Ulnar Kneading"
      - id: knuckle_knead
        label: "Knuckle Kneading"
        output: "- Knuckle Kneading"
      - id: fingertip_knead
        label: "Fingertip Kneading"
        output: "- Fingertip Kneading"

  - id: pecs_supine
    label: "UPPER CHEST WALL / PECS (Supine)"
    items:
      - id: myofascial_pec
        label: "Myofascial Techniques (Pectoralis Major)"
        output: "- Myofascial Techniques (Pectoralis Major)"
      - id: pnf_pec
        label: "PNF Stretch (Pectoralis Major & Minor)"
        output: "- Proprioceptive Neuromuscular Facilitation (PNF) Stretch (Pectoralis Major & Minor)"
      - id: spec_comp_subscap
        label: "Specific Compressions (Subscapularis)"
        output: "- Specific Compressions (Subscapularis)"
      - id: pin_active_subscap
        label: "Pin & Active Release (Subscapularis)"
        output: "- Pin & Active Release (Subscapularis)"

  - id: arms_supine
    label: "ARMS (Supine)"
    items:
      - id: swedish
        label: "General Swedish Techniques"
        output: "- General Swedish Techniques"
      - id: pin_stretch_bicep
        label: "Pin & Stretch (Biceps Brachii, Brachialis)"
        output: "- Pin & Stretch (Biceps Brachii, Brachialis)"
      - id: pin_active_ext
        label: "Pin & Active Release (Extensor Digitorum, Extensor Carpi Ulnaris)"
        output: "- Pin & Active Release (Extensor Digitorum, Extensor Carpi Ulnaris)"
      - id: spec_comp_ext
        label: "Specific Compressions (Extensor Digitorum, Extensor Carpi Ulnaris)"
        output: "- Specific Compressions (Extensor Digitorum, Extensor Carpi Ulnaris)"
      - id: jt_mob_wrist
        label: "Joint Mobilizations (Wrist, Elbow)"
        output: "- Joint Mobilizations (Wrist, Elbow)"

  - id: hns_supine
    label: "HEAD, NECK, & SHOULDERS (Supine)"
    items:
      - id: swedish
        label: "General Swedish Techniques"
        output: "- General Swedish Techniques"
      - id: pin_stretch_suboccip
        label: "Pin & Stretch (Suboccipital Muscles)"
        output: "- Pin & Stretch (Suboccipital Muscles)"
      - id: spec_comp_scalene_scm
        label: "Specific Compressions (Scalene Muscles, SCM)"
        output: "- Specific Compressions (Scalene Muscles, SCM)"
      - id: picking_up_scm
        label: "Picking Up (SCM)"
        output: "- Picking Up (SCM)"
      - id: traction_breathing
        label: "Traction Neck & Diaphragmatic Breathing Cues"
        output: "- Traction Neck & Diaphragmatic Breathing Cues"
      - id: myofascial_scm
        label: "Myofascial Techniques (over SCM)"
        output: "- Myofascial Techniques (over SCM)"
      - id: spec_comp_tmj
        label: "Specific Compressions (Masseter, Temporalis - TMJ)"
        output: "- Specific Compressions (Scalene Muscles, SCM, Masseter, Temporalis)"
      - id: myofascial_skull
        label: "Myofascial Techniques (Frontal, Temporal bone)"
        output: "- Myofascial Techniques (Frontal bone, Temporal bone)"
      - id: broad_comp_skull
        label: "Broad Compressions (Parietal, Temporal bone)"
        output: "- Broad Compressions (Parietal bone, Temporal bone)"
```

Notes on tx_regions.yml migration:
- The old `entries:` root key and `BlockSelectFile` struct are gone; replaced with `lists:`.
- The old `header:` field on each entry (e.g. `"#### UPPER, MIDDLE & LOWER BACK (Prone)"`) is NOT carried to `HierarchyList` because `HierarchyList` has no `header` or `preview` field in the current struct. The block_select renderer currently uses `BlockSelectGroup.header` to print the region heading. The shim in ST4 must decide how to source this value; the most natural approach is to derive the header from `list.label` in the shim (e.g. `format!("#### {}", list.label)`). This is a shim-level concern and does not block the YAML migration.
- Item `id` and `label` are both required by `HierarchyItem`. All item IDs are preserved from the original `PartOption::Full` ids.
- `default: false` is preserved only where explicitly set in the source (fascial_l4l5 in back_lower_prone). All other items omit `default:` (the renderer's own default applies).
- Items that shared the same `id` across different lists (e.g. `broad_comp`, `swedish`, `ulnar_knead`) are fine because items are not registered in the typed ID registry - only lists, fields, sections, and groups are.

### Step 4: Migrate boilerplate.yml

Replace the entire contents of `/c/scribble/data/boilerplate.yml` with the following:

```yaml
boilerplate:
  - id: treatment_plan_disclaimer
    text: |
      Regions and locations are bilateral unless indicated otherwise.
      Patient is pillowed under ankles when prone, and under knees when supine.
  - id: informed_consent
    text: Patient has been informed of the risks and benefits of massage therapy, and has given informed consent to assessment and treatment.
```

Notes on boilerplate.yml migration:
- Old `blocks:` root key and `type: boilerplate` discriminants are gone.
- The `boilerplate:` top-level key maps directly to `HierarchyFile.boilerplate: Vec<BoilerplateEntry>`.
- `id` and `text` fields are unchanged.
- The `|` block scalar on `treatment_plan_disclaimer` preserves the two-line text with a trailing newline, identical to the original.

### Step 5: Migrate objective_findings.yml, remedial.yml, infection_control.yml

Each of these files currently uses `blocks: - type: options-list` format with `entries:` containing objects that have only `label:` and `output:` fields (no `id`). The `HierarchyItem` struct requires `id: String` as a mandatory field. Each item must be assigned a stable `id` derived from the label.

**ID derivation convention:** convert the label to lowercase, replace spaces and special characters with underscores, truncate to avoid duplicates, and keep it short but recognizable. The exact IDs chosen are not semantically meaningful to the runtime (items are not referenced by ID across files) but must be unique within their list.

Replace `/c/scribble/data/objective_findings.yml` with:

```yaml
lists:
  - id: objective_findings
    label: "Objective Findings"
    items:
      - id: bl_trap_uf_irmt
        label: "BL Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "BL Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - id: bl_trap_uf_l_r_irmt
        label: "BL (L > R) Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "BL (L > R) Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - id: bl_trap_uf_r_l_irmt
        label: "BL (R > L) Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "BL (R > L) Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - id: l_trap_uf_irmt
        label: "L Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "L Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - id: r_trap_uf_irmt
        label: "R Trapezius (Upper Fibers): Increased Resting Muscle Tension"
        output: "R Trapezius (Upper Fibers): Increased Resting Muscle Tension"
      - id: rhomboids_irmt
        label: "Rhomboids: Increased Resting Muscle Tension"
        output: "Rhomboids: Increased Resting Muscle Tension"
      - id: bl_ls_irmt
        label: "BL Levator Scapula: Increased Resting Muscle Tension"
        output: "BL Levator Scapula: Increased Resting Muscle Tension"
      - id: bl_es_irmt
        label: "BL Erector Spinae: Increased Resting Muscle Tension"
        output: "BL Erector Spinae: Increased Resting Muscle Tension"
      - id: bl_ql_l_r_tight
        label: "BL (L > R) Quadratus Lumborum: Tightness & Tender palpated"
        output: "BL (L > R) Quadratus Lumborum: Tightness & Tender palpated"
      - id: bl_ql_irmt
        label: "BL Quadratus Lumborum: Increased Resting Muscle Tension"
        output: "BL Quadratus Lumborum: Increased Resting Muscle Tension"
      - id: bl_pir_irmt
        label: "BL Piriformis: Increased Resting Muscle Tension"
        output: "BL Piriformis: Increased Resting Muscle Tension"
      - id: bl_gmed_irmt
        label: "BL Gluteus Medius: Increased Resting Muscle Tension"
        output: "BL Gluteus Medius: Increased Resting Muscle Tension"
      - id: bl_scalene_irmt
        label: "BL Scalene Muscles: Increased Resting Muscle Tension"
        output: "BL Scalene Muscles: Increased Resting Muscle Tension"
      - id: bl_scm_irmt
        label: "BL SCM: Increased Resting Muscle Tension"
        output: "BL Sternocleidomastoid (SCM): Increased Resting Muscle Tension"
      - id: bl_suboccip_irmt
        label: "BL Suboccipital Group: Increased Resting Muscle Tension"
        output: "BL Suboccipital Muscle Group: Increased Resting Muscle Tension"
      - id: bl_pec_irmt
        label: "BL Pectoralis Major: Increased Resting Muscle Tension"
        output: "BL Pectoralis Major: Increased Resting Muscle Tension"
      - id: bl_tfl_irmt
        label: "BL TFL: Increased Resting Muscle Tension"
        output: "BL TFL (Tensor Fasciae Latae): Increased Resting Muscle Tension"
      - id: bl_ham_irmt
        label: "BL Hamstrings: Increased Resting Muscle Tension"
        output: "BL Hamstrings: Increased Resting Muscle Tension"
      - id: bl_gastroc_irmt
        label: "BL Gastrocnemius: Increased Resting Muscle Tension"
        output: "BL Gastrocnemius: Increased Resting Muscle Tension"
      - id: hypertonicity
        label: "Hypertonicity palpated"
        output: "Hypertonicity palpated"
      - id: tightness_tenderness
        label: "Tightness & Tenderness palpated"
        output: "Tightness & Tender palpated"
      - id: trigger_points
        label: "Trigger points palpated"
        output: "Trigger points palpated"
      - id: restricted_rom
        label: "Restricted ROM observed"
        output: "Restricted Range of Motion observed"
      - id: protective_guarding
        label: "Protective guarding observed"
        output: "Protective guarding observed"
```

Replace `/c/scribble/data/remedial.yml` with:

```yaml
lists:
  - id: remedial_exercises
    label: "Remedial Exercises & Self-Care"
    items:
      - id: psoas_kneeling_stretch
        label: "PSOAS KNEELING STRETCH - Low Back Pain"
        output: "PSOAS KNEELING STRETCH - Lengthen Psoas to Reduce Low Back Pain:\n- - \"Slide hips all the way forwards, then rotate all the way over the top knee.\n- - Slight lean back, and disco point to the back corner.\"\n- - 60s static stretch, bilaterally (30s-120s).\n- - 5 / week, for 2 weeks or until next appointment."
      - id: wall_angel
        label: "WALL ANGEL - Rhomboid Strength & Postural Awareness"
        output: "WALL ANGEL - Increase Rhomboid Strength & Postural Awareness:\n- - 5->10 Reps, daily at work, for 3 weeks or until next appointment.\n- - \"Perfect reps, no rushing!\""
      - id: cactus_arms
        label: "CACTUS ARMS (Angel Wings) - Rhomboid Strength"
        output: "CACTUS ARMS (Angel Wings) - Increase Rhomboid Strength & Postural Awareness:\n- - 5 Reps, daily at work, for 3 weeks or until next appointment."
      - id: sphinx_stretch
        label: "SPHINX STRETCH - Lumbar Mobility"
        output: "SPHINX STRETCH / HOLDS - Improve Lumbar Mobility:\n- - 8-12 Reps (hold 5s) x 1 Set.\n- - 3 / week, for 2 weeks or until next appointment."
      - id: rec_fem_kneeling_stretch
        label: "RECTUS FEMORIS KNEELING WALL STRETCH - Knee and Calf Tightness"
        output: "RECTUS FEMORIS KNEELING WALL STRETCH - Lengthen Rec Fem to Reduce Knee and Calf Tightness:\n- - \"Kneel against the base of the wall, then rotate one foot up.\n- - Keeping the foot against the wall, slowly raise your torso vertically to stretch.\"\n- - 60s static stretch, bilaterally (30s-120s).\n- - 2-4 / week, for 6 weeks or until next appointment."
      - id: rainbow_arc_scm
        label: "RAINBOW ARC SCM STRETCH - Lengthen SCM"
        output: "RAINBOW ARC SCM STRETCH - Lengthen Sternocleidomastoid (SCM):\n- - \"Laterally flex your neck, then make an arc with your nose from front, upwards, and back.\n- - To intensify the stretch, reach down with one arm.\"\n- - 30s-60s dynamic stretch, bilaterally (60s-120s total).\n- - 1-2 daily, or as needed."
      - id: glute_bridges
        label: "GLUTE BRIDGES - Gluteal Strength, Core Stability"
        output: "GLUTE BRIDGES - Increase Gluteal Strength, Core Stability:\n- - 12 Reps (hold 3 sec) x 3 Sets (Rest 60s).\n- - \"Squeeze the glutes!\"\n- - 3 / week, for 6 weeks or until next appointment."
      - id: bw_squats
        label: "BODYWEIGHT SQUATS - Gluteal Strength"
        output: "BODYWEIGHT SQUATS - Increase Gluteal Strength:\n- - 8->12 Reps x 3 Sets (Rest 60s), using 80% strength.\n- - 2 / week, for 6 weeks or until next appointment."
      - id: dumbbell_row
        label: "BENT-OVER DUMBBELL ROW - Rhomboid Strength"
        output: "BENT-OVER DUMBBELL ROW - Increase Rhomboid Strength:\n- - 8-10 Reps x 3 Sets (Rest 60s), using 80% strength.\n- - 4 / week, for 3 weeks or until next appointment."
      - id: regular_walks
        label: "REGULAR WALKS - Daily Physical Activity"
        output: "REGULAR WALKS - Increase Daily Physical Activity:\n- - As early in the day as possible, go outside for a daily walk.\n- - Even 5 minutes is a great start! Do longer if you can, and avoid skipping entirely."
      - id: return_to_exercise
        label: "Recommend returning to regular exercise habits."
        output: "Recommend returning to regular exercise habits."
      - id: continue_physio
        label: "Continue exercise program as directed by physiotherapist."
        output: "Continue exercise program as directed by physiotherapist."
      - id: continue_chiro
        label: "Continue exercise program as directed by chiropractor."
        output: "Continue exercise program as directed by chiropractor."
      - id: no_goals
        label: "No specific exercise or self-care goals discussed."
        output: "No specific exercise or self-care goals discussed."
```

Replace `/c/scribble/data/infection_control.yml` with:

```yaml
lists:
  - id: infection_control
    label: "Infection Control"
    items:
      - id: disinfect_contact_points
        label: "Disinfected patient contact points before and after appointment (door knob, coat hooks, patient chair)."
        output: "Disinfected patient contact points before and after appointment (door knob, coat hooks, patient chair)."
      - id: disinfect_equipment
        label: "Disinfected equipment before and after appointment (massage table, face cradle, RMT rolling stool, oil bottle)."
        output: "Disinfected equipment before and after appointment (massage table, face cradle, RMT rolling stool, oil bottle)."
      - id: clean_linens
        label: "Clean Linens used (fitted sheet, flat sheet, face cradle cover, blanket)."
        output: "Clean Linens used (fitted sheet, flat sheet, face cradle cover, blanket)."
      - id: washed_hands
        label: "RMT washed hands before and after treatment."
        output: "RMT washed hands before and after treatment."
      - id: mask_worn
        label: "Mask worn by RMT."
        output: "Mask worn by RMT."
```

Notes on list-type file migration:
- Old `blocks: - type: options-list id: ... entries:` structure is gone; replaced by `lists: - id: ... items:`.
- Each list now has a single `HierarchyList` with `id` and `items`. The list id matches the section it belongs to (`objective_findings`, `remedial_exercises`, `infection_control`) so the shim can locate the right list when loading via `data_file:` dispatch.
- Every item gets an `id:` field (required by `HierarchyItem`). IDs are stable identifiers derived from content; they are not user-visible and not referenced across files.
- `label:` and `output:` are preserved exactly. Where the original had `label != output` (e.g. BL SCM, BL TFL, restricted ROM), both fields are kept distinct.

### Step 6: Verify all files parse

After writing all 6 files, run:

```
cargo test
```

The existing `lower_back_prone_fascial_l4l5_starts_unselected` test at `src/data.rs` currently parses `tx_regions.yml` as `BlockSelectFile`. After migration the file uses `lists:` at the top level, which `BlockSelectFile` cannot parse - the test WILL fail and MUST be updated as part of this step. Replace it with:

```rust
    #[test]
    fn lower_back_prone_fascial_l4l5_starts_unselected() {
        let yaml_content = include_str!("../data/tx_regions.yml");
        let file: HierarchyFile =
            serde_yaml::from_str(yaml_content).expect("tx_regions.yml must parse as HierarchyFile");

        let lists = file.lists.as_deref().unwrap_or(&[]);
        let region = lists
            .iter()
            .find(|l| l.id == "back_lower_prone")
            .expect("back_lower_prone list must exist in tx_regions.yml");

        let fascial_entry = region
            .items
            .iter()
            .find(|i| i.id == "fascial_l4l5")
            .expect("fascial_l4l5 item must exist in back_lower_prone");

        assert_eq!(
            fascial_entry.default,
            Some(false),
            "fascial_l4l5 in LOWER BACK (Prone) must have default: Some(false)"
        );
    }
```

Also verify with a one-off Rust snippet or by examining `load_hierarchy_dir` test output that the real `data/` directory parses successfully.

## Verification

### Manual tests

- After migration, run `cargo build` and confirm it compiles cleanly.
- Run `grep -r "map_label:" /c/scribble/data/` and confirm zero results (mission test criterion).
- Run `grep -r "type:" /c/scribble/data/` and confirm no `type: group`, `type: section`, `type: field`, `type: options-list`, or `type: boilerplate` lines remain.

### Automated tests

- `cargo test` must pass with zero failures. The expected test count after sub-tasks 1 and 2 is 196; after this sub-task the `lower_back_prone_fascial_l4l5_starts_unselected` test must be updated to use `HierarchyFile` parse (if it has not already been updated), keeping the total count the same or higher.
- Mission test criterion: `tx_regions.yml` parsed as `HierarchyFile` has `back_lower_prone` list with `fascial_l4l5` item having `default: Some(false)` - the existing or updated test covers this.
- Mission test criterion: `load_hierarchy_dir` against real `data/` returns `Ok` - covered by the integration test that will be written in ST4; for this sub-task, manual `cargo build` success is sufficient.
- Mission test criterion: all 6 data YAML files parse as `HierarchyFile` without error - verified by `cargo test` passing (the loader reads all files in `data/`).

## Changelog

### Review - 2026-04-03
- #1: Removed all `map_label:` keys from sections.yml YAML content in Step 2 (15 occurrences across all sections). The BRIEF manual test criterion requires `grep -r "map_label:" /c/scribble/data/` to return zero results; keeping `map_label:` in the migrated YAML directly contradicted that criterion. Updated the Step 2 notes bullet to accurately state that `map_label:` is removed and the shim derives `SectionConfig.map_label` from `section.nav_label`.

### Review - 2026-04-03
- #2: Added `HierarchyField` extension diff to Step 1 (`composite: Option<CompositeConfig>`, `default: Option<String>`, `repeat_limit: Option<usize>`). The prose at the end of Step 2 stated these should be added in Step 1 "for completeness" but the Step 1 code block omitted them, leaving an implementer who reads only the diff without guidance.
- #3: Replaced conditional Step 6 test-update note with a required rewrite and provided the full replacement Rust test for `lower_back_prone_fascial_l4l5_starts_unselected` using `HierarchyFile` instead of `BlockSelectFile`. The old test WILL break after migration (the file no longer parses as `BlockSelectFile`), so the update is mandatory, not conditional. Updated Step 2 notes note and Step 6 prose accordingly.

## Progress
- Step 1: Extended HierarchySection (added serde(default) on map_label, data_file, heading_search_text, heading_label, note_render_slot, is_intake), HierarchyGroup (added num), and HierarchyField (added composite, default, repeat_limit) in src/data.rs
- Step 2: Replaced sections.yml with HierarchyFile format (template/groups/sections top-level keys, nav_label instead of name, no map_label keys)
- Step 3: Replaced tx_regions.yml with HierarchyFile format (lists with items, no entries/header keys)
- Step 4: Replaced boilerplate.yml with HierarchyFile format (boilerplate top-level key, no blocks/type keys)
- Step 5: Replaced objective_findings.yml, remedial.yml, infection_control.yml with HierarchyFile format (lists with items including stable ids)
- Step 6: Updated lower_back_prone_fascial_l4l5_starts_unselected test to use HierarchyFile instead of BlockSelectFile; added hierarchy fallback in load_data_dir so existing tests pass via hierarchy_to_app_data shim; all 196 tests pass

## Implementation
Complete - 2026-04-03
