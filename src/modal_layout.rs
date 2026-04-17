// Unit arrangement types and layout logic for the modal panel.
// Extracted from modal.rs so this layer can evolve independently.

pub const MODAL_HEIGHT_RATIO: f32 = 0.8;
const MODAL_CHROME_HEIGHT: f32 = 80.0;
const MODAL_ROW_HEIGHT: f32 = 28.0;

#[derive(Debug, Clone, PartialEq)]
// LESSON: Tracks which part of a modal panel the keyboard controls. Exists so keyboard events can be routed to either the search bar or the list depending on where focus is.
pub enum ModalFocus {
    SearchBar,
    List,
}

#[derive(Debug, Clone, PartialEq)]
// LESSON: A snapshot of one modal panel's complete state at a single moment. Exists so the renderer has everything it needs to draw the panel without reaching into app state directly.
pub struct ModalListViewSnapshot {
    pub title: String,
    pub query: String,
    pub rows: Vec<String>,
    pub filtered: Vec<usize>,
    pub list_cursor: usize,
    pub list_scroll: usize,
    pub focus: ModalFocus,
}

#[derive(Debug, Clone, PartialEq)]
// LESSON: The full ordered list of modal panels for one note-filling session, plus which panel is currently active. Exists because filling out a note requires moving through multiple panels in sequence.
pub struct SimpleModalSequence {
    pub snapshots: Vec<ModalListViewSnapshot>,
    pub active_sequence_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
// LESSON: Describes one screen-width group of panels as a start/end index range into the sequence. Exists because not all panels fit side-by-side at once, so they are divided into units that fit.
pub struct ModalUnitRange {
    pub start: usize,
    pub end: usize,
    pub shows_stubs: bool,
}

#[derive(Debug, Clone, PartialEq)]
// LESSON: The complete geometry result: the full sequence plus all computed unit groupings. Geometry only - which unit is currently active is owned by AppState, not this struct.
pub struct SimpleModalUnitLayout {
    pub sequence: SimpleModalSequence,
    pub units: Vec<ModalUnitRange>,
    // active_unit_index is NOT stored here; AppState.active_unit_index is the single
    // canonical source of truth. This struct is geometry-only.
}

// Small fits 25 cpl - the floor comes from word-length distributions across languages,
// meaning even short single-word lists won't be clipped. These are often single-word fields anyway.
// Large fits 85 cpl - matched to the single-line length on the clinic platforms this app targets,
// so a full clinic row can be displayed without wrapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalSize {
    Small,
    Medium,
    Large,
}

impl ModalSize {
    fn dimensions(self) -> (f32, f32) {
        match self {
            Self::Small => (280.0, 200.0),
            Self::Medium => (560.0, 360.0),
            Self::Large => (720.0, 480.0),
        }
    }
}

// LESSON 3 Stubs: The four action markers shown at the edges of a unit (navigate left/right, exit, confirm). Exists so the renderer knows which symbol to draw and what each edge card means.
/// The semantic kind of a stub card shown at the edge of a modal unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalStubKind {
    NavLeft,  // "<" - navigate to the previous unit
    NavRight, // ">" - navigate to the next unit
    Exit,     // "-" - leave the field without adding to the note
    Confirm,  // "+" - add to note / complete the field
}

impl ModalStubKind {
    /// The single character displayed centered inside the stub card.
    pub fn symbol(self) -> char {
        match self {
            Self::NavLeft => '<',
            Self::NavRight => '>',
            Self::Exit => '-',
            Self::Confirm => '+',
        }
    }
}

fn modal_size_for_snapshot(snapshot: &ModalListViewSnapshot) -> ModalSize {
    let max_chars = std::iter::once(snapshot.title.chars().count())
        .chain(std::iter::once(snapshot.query.chars().count()))
        .chain(snapshot.rows.iter().map(|row| row.chars().count()))
        .max()
        .unwrap_or(0);
    if max_chars <= 25 {
        ModalSize::Small
    } else if max_chars <= 66 {
        ModalSize::Medium
    } else {
        ModalSize::Large
    }
}

pub fn modal_list_view_dimensions(snapshot: &ModalListViewSnapshot) -> (f32, f32) {
    modal_size_for_snapshot(snapshot).dimensions()
}

// LESSON 4: Two prep calculations before packing: caps the gap size at 0.5% of viewport width, then subtracts stub and spacer space from both sides to get the usable content width. Without these, the packing algorithm would have no idea how much room it actually has.
/// Returns the effective spacer width for a given viewport axis dimension.
/// Caps at 2% of the viewport to prevent oversized gaps on wide displays.
pub fn effective_spacer_width(viewport_width: f32, modal_spacer_width: f32) -> f32 {
    (viewport_width * 0.005).min(modal_spacer_width)
}

/// Returns the bounding box available for full-width modals within a unit.
/// Reserves space for one stub and one spacer on each side.
/// Clamped to zero so extremely narrow viewports never produce a negative content limit.
pub fn unit_bounding_box(viewport_width: f32, stub_width: f32, spacer_width: f32) -> f32 {
    (viewport_width - 2.0 * (stub_width + spacer_width)).max(0.0)
}

// LESSON 5: The main packing function. Takes all panels and greedily groups them into screen-fitting units, left to right. If the viewport is unknown it falls back to one unit containing everything. If a panel is wider than the bounding box it gets the full viewport width and stubs are hidden.
pub fn build_simple_modal_unit_layout(
    sequence: SimpleModalSequence,
    viewport_width: Option<f32>,
    spacer_width: f32,
    stub_width: f32,
) -> Option<SimpleModalUnitLayout> {
    if sequence.snapshots.is_empty() {
        return None;
    }

    let viewport_width = viewport_width
        .filter(|width| width.is_finite() && *width > 0.0)
        .unwrap_or(f32::INFINITY);
    let raw_spacer = spacer_width.max(0.0);
    let stub_width = stub_width.max(0.0);
    let spacer_width = effective_spacer_width(viewport_width, raw_spacer);
    let bounding_box = unit_bounding_box(viewport_width, stub_width, spacer_width);
    let widths = sequence
        .snapshots
        .iter()
        .map(|snapshot| modal_list_view_dimensions(snapshot).0)
        .collect::<Vec<_>>();

    let mut units = Vec::new();
    let mut start = 0usize;
    while start < widths.len() {
        let first_width = widths[start];
        let first_exceeds_bounding_box = first_width > bounding_box;
        let shows_stubs = !first_exceeds_bounding_box;
        let content_limit = if first_exceeds_bounding_box {
            viewport_width
        } else {
            bounding_box
        };
        let mut used_width = if content_limit.is_finite() {
            first_width.min(content_limit)
        } else {
            first_width
        };
        let mut end = start;

        while end + 1 < widths.len() {
            let next_width = widths[end + 1];
            let next_used_width = used_width + spacer_width + next_width;
            if next_used_width <= content_limit {
                end += 1;
                used_width = next_used_width;
            } else {
                break;
            }
        }

        units.push(ModalUnitRange {
            start,
            end,
            shows_stubs,
        });
        start = end + 1;
    }

    Some(SimpleModalUnitLayout {
        sequence,
        units,
    })
}

// LESSON 6: Two height helpers - one sets modal height to 80% of the viewport (with a 160px floor), the other calculates how many list rows fit after subtracting the chrome. Vertical position is handled elsewhere; this is only about size.
pub fn modal_height_for_viewport(viewport_height: Option<f32>, fallback_height: f32) -> f32 {
    viewport_height
        .map(|height| (height * MODAL_HEIGHT_RATIO).max(160.0))
        .unwrap_or(fallback_height)
}

pub fn modal_window_size_for_height(modal_height: f32, hint_count: usize) -> usize {
    let hint_cap = hint_count.max(1);
    let available_rows = ((modal_height - MODAL_CHROME_HEIGHT) / MODAL_ROW_HEIGHT)
        .floor()
        .max(1.0) as usize;
    available_rows.min(hint_cap)
}

#[cfg(test)]
mod modal_sizing_tests {
    use super::*;

    #[test]
    fn modal_height_uses_eighty_percent_of_viewport_when_known() {
        assert_eq!(modal_height_for_viewport(Some(1000.0), 360.0), 800.0);
    }

    #[test]
    fn modal_height_uses_fallback_before_resize_event() {
        assert_eq!(modal_height_for_viewport(None, 360.0), 360.0);
    }

    #[test]
    fn modal_window_size_is_capped_by_hint_count() {
        assert_eq!(modal_window_size_for_height(1000.0, 12), 12);
    }

    #[test]
    fn modal_window_size_shrinks_for_short_viewports() {
        assert_eq!(modal_window_size_for_height(192.0, 12), 4);
    }

    #[test]
    fn simple_modal_unit_layout_groups_snapshots_within_bounding_box() {
        let layout = build_simple_modal_unit_layout(
            SimpleModalSequence {
                snapshots: vec![
                    ModalListViewSnapshot {
                        title: "One".to_string(),
                        query: String::new(),
                        rows: vec!["A".to_string()],
                        filtered: vec![0],
                        list_cursor: 0,
                        list_scroll: 0,
                        focus: ModalFocus::List,
                    },
                    ModalListViewSnapshot {
                        title: "Two".to_string(),
                        query: String::new(),
                        rows: vec!["B".to_string()],
                        filtered: vec![0],
                        list_cursor: 0,
                        list_scroll: 0,
                        focus: ModalFocus::List,
                    },
                    ModalListViewSnapshot {
                        title: "Three".to_string(),
                        query: String::new(),
                        rows: vec!["C".to_string()],
                        filtered: vec![0],
                        list_cursor: 0,
                        list_scroll: 0,
                        focus: ModalFocus::List,
                    },
                ],
                active_sequence_index: 2,
            },
            Some(940.0),
            18.0,
            120.0,
        )
        .expect("layout should build");

        assert_eq!(
            layout.units,
            vec![
                ModalUnitRange {
                    start: 0,
                    end: 1,
                    shows_stubs: true,
                },
                ModalUnitRange {
                    start: 2,
                    end: 2,
                    shows_stubs: true,
                },
            ]
        );
        // active_unit_index is now owned by AppState, not the layout.
        // The sequence.active_sequence_index (2) belongs to units[1] (start=2, end=2).
        assert!(layout.units[1].start == 2);
    }

    #[test]
    fn simple_modal_unit_layout_omits_stubs_for_oversized_first_modal() {
        let layout = build_simple_modal_unit_layout(
            SimpleModalSequence {
                snapshots: vec![ModalListViewSnapshot {
                    title: "Oversized".to_string(),
                    query: String::new(),
                    rows: vec!["X".repeat(120)],
                    filtered: vec![0],
                    list_cursor: 0,
                    list_scroll: 0,
                    focus: ModalFocus::List,
                }],
                active_sequence_index: 0,
            },
            Some(700.0),
            20.0,
            120.0,
        )
        .expect("layout should build");

        assert_eq!(
            layout.units,
            vec![ModalUnitRange {
                start: 0,
                end: 0,
                shows_stubs: false,
            }]
        );
    }

    #[test]
    fn effective_spacer_width_caps_at_half_percent_of_viewport() {
        assert_eq!(super::effective_spacer_width(1000.0, 40.0), 5.0);
        assert_eq!(super::effective_spacer_width(500.0, 40.0), 2.5);
        assert_eq!(super::effective_spacer_width(100.0, 40.0), 0.5);
    }

    #[test]
    fn effective_spacer_width_does_not_exceed_theme_value() {
        // 0.5% of 200 = 1.0, but theme cap is 3.0
        assert_eq!(super::effective_spacer_width(200.0, 3.0), 1.0);
    }

    #[test]
    fn unit_bounding_box_subtracts_both_stubs_and_spacers() {
        assert_eq!(super::unit_bounding_box(1200.0, 120.0, 20.0), 920.0);
    }

    #[test]
    fn unit_bounding_box_clamps_to_zero_on_narrow_viewport() {
        assert_eq!(super::unit_bounding_box(50.0, 120.0, 20.0), 0.0);
    }
}
